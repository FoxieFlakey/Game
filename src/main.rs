#![feature(unsafe_cell_access)]
#![feature(current_thread_id)]

use std::{error::Error, pin::Pin, rc::Rc, sync::atomic::{AtomicBool, AtomicU32, Ordering}, task::Poll, time::{Duration, Instant}};

use futures::{FutureExt, poll};
use tokio::{signal::unix::{SignalKind, signal}, sync::Notify};

use crate::{local_resource::LocalResource, util::{ErrorWithContext, StringError, sig_safe}, window::Window};

mod local_resource;
mod logging;
mod runtimes;
mod rendering;
mod util;
mod window;
mod states;

// Maximum of interrupts before hard quit triggered
const WARN_INTERRUPT_COUNT: u32 = 3;
const MAX_INTERRUPT_COUNT: u32 = 5;
static INTERRUPT_COUNT: AtomicU32 = AtomicU32::new(0);

// Different counter for emergency sigint, which bypasses
// game loop, directly handled in signal handler
const WARN_SIG_INTERRUPT_COUNT: u32 = 8;
const MAX_SIG_INTERRUPT_COUNT: u32 = 15;
static SIG_INTERRUPT_COUNT: AtomicU32 = AtomicU32::new(0);

// Give graceful 10 seconds for shutting down
const GRACEFUL_SHUTDOWN_MILISEC: u64 = 10000;
static IS_SHUTTING_DONW: AtomicBool = AtomicBool::new(false);

fn handle_sig_term() {
    if !IS_SHUTTING_DONW.load(Ordering::Relaxed) {
        return
    }

    sig_safe::write_str_to_stdout("[Signal Handler] Main thread didnt cleanup in time before next SIGTERM, hard quitting\n");
    sig_safe::exit(1);
}

fn handle_sig_hardquit() {
    let current_interrupts = SIG_INTERRUPT_COUNT.fetch_add(1, Ordering::Relaxed).saturating_add(1);
    if current_interrupts >= MAX_SIG_INTERRUPT_COUNT {
        sig_safe::write_str_to_stdout("[Signal handler] Main thread does not respond to signal after ");
        let mut buf = itoa::Buffer::new();
        sig_safe::write_str_to_stdout(buf.format(MAX_SIG_INTERRUPT_COUNT));
        sig_safe::write_str_to_stdout(" interrupts. Hard quitting now\n");

        sig_safe::exit(1);
    }
    
    if current_interrupts >= WARN_SIG_INTERRUPT_COUNT {
        sig_safe::write_str_to_stdout("[Signal handler] Main thread does not respond to signals, hard quiting via signal in ");
        let mut buf = itoa::Buffer::new();
        sig_safe::write_str_to_stdout(buf.format(MAX_SIG_INTERRUPT_COUNT.saturating_sub(current_interrupts)));
        sig_safe::write_str_to_stdout(" interrupts\n");
    }
}

fn main() {
    logging::init();
    crate::info!("Hello, world!");
    
    // Installing quit handling signal early
    // SAFETY: The handler is only calls async-signal-safe functions and does not panics
    if let Err(e) = unsafe { signal_hook::low_level::register(signal_hook::consts::SIGINT, handle_sig_hardquit) } {
        crate::fatal!("Cannot install SIGINT handler: {e}");
        return
    }
    
    // SAFETY: The handler is only calls async-signal-safe functions and does not panics
    if let Err(e) = unsafe { signal_hook::low_level::register(signal_hook::consts::SIGTERM, handle_sig_term) } {
        crate::fatal!("Cannot install SIGTERM handler: {e}");
        return
    }

    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(x) => {
            runtimes::main::set(x);
            let res = runtimes::main::get().block_on(async {
                let mut sigint = signal(SignalKind::interrupt())
                    .map_err(|e| ErrorWithContext::with_message("Cannot install SIGINT handler", Box::new(e)))?;
                let mut sigterm = signal(SignalKind::terminate())
                    .map_err(|e| ErrorWithContext::with_message("Cannot install SIGTERM handler", Box::new(e)))?;
                let quit_notifier = Rc::new(Notify::new());

                let main_future = async_main(|| {
                    let inner_notifier = quit_notifier.clone();
                    Box::pin(async move { inner_notifier.notified().await })
                });
                tokio::pin!(main_future);

                loop {
                    tokio::select! {
                        _ = sigint.recv() => {
                            let current_interrupts = INTERRUPT_COUNT.fetch_add(1, Ordering::Relaxed).saturating_add(1);
                            if current_interrupts >= MAX_INTERRUPT_COUNT {
                                alert!("Unresponsive main loop. Hard quitting now, interrupts exceeds {MAX_INTERRUPT_COUNT}");
                                break;
                            }
                            
                            if current_interrupts >= WARN_INTERRUPT_COUNT {
                                alert!("Unresponsive main loop. Hard quit will trigger in {} signal count", MAX_INTERRUPT_COUNT.saturating_sub(current_interrupts));
                            }

                            if current_interrupts == 1 {
                                alert!("SIGINT or Ctrl-C received starting orderly shutdown...");
                            }
                            quit_notifier.notify_waiters();
                        }
                        _ = sigterm.recv() => {
                            // Sigterm works like trigger one quit and starts timer
                            // if timer exhausted and program hasn't cleaned up
                            // perform hard exits anyway.
                            // 
                            // SIGINT is not handled in here
                            let deadline = Instant::now() + Duration::from_millis(GRACEFUL_SHUTDOWN_MILISEC);
                            alert!("SIGTERM received starting orderly shutdown with deadline in {GRACEFUL_SHUTDOWN_MILISEC} ms");
                            
                            IS_SHUTTING_DONW.store(true, Ordering::Relaxed);
                            quit_notifier.notify_waiters();

                            tokio::select! {
                                ret = &mut main_future => {
                                    return ret;
                                }

                                _ = tokio::time::sleep_until(deadline.into()) => {
                                    fatal!("Program did not shutdown in {GRACEFUL_SHUTDOWN_MILISEC} ms");
                                }
                            }
                            break;
                        }
                        ret = &mut main_future => {
                            // Game done running
                            return ret;
                        }
                    }
                }

                Err(ErrorWithContext::new_err(StringError::from("Hard quit triggered".to_string())))
            });
            if let Err(e) = res {
                crate::fatal!("Cannot run game: {e}");
            }

            crate::info!("Game quited! bye bye UwU");
        }

        Err(e) => {
            crate::fatal!("Error initializing tokio main runtime: {e}");
        }
    }
}

pub struct SdlState {
  sdl: sdl3::Sdl,
  audio: sdl3::AudioSubsystem,
  video: sdl3::VideoSubsystem,
  event: sdl3::EventSubsystem,
  event_pump: sdl3::EventPump
}

pub struct MainState {
    window: Window
}

struct Resources {
    sdl_resource: LocalResource<SdlState>,
    main_resource: LocalResource<MainState>
}

impl Resources {
    async fn poll_loop(&mut self) -> ! {
        tokio::join!(
            self.sdl_resource.poll_loop(),
            self.main_resource.poll_loop()
        );
        
        panic!("Something horribly gone wrong, poll loop for resources intended to never finishes");
    }
}

async fn init() -> Result<Resources, ErrorWithContext<dyn Error + 'static>> {
    rendering::init()?;

    let sdl = sdl3::init()
            .map_err(|x| ErrorWithContext::with_message("Cannot initialize SDL3", Box::new(x)))?;
    let (sdl_resource, accessor) = LocalResource::new("SDL subsystems", SdlState {
        event: sdl.event()
            .map_err(|x| ErrorWithContext::with_message("Cannot initialize events subsystem", Box::new(x)))?,
        video: sdl.video()
            .map_err(|x| ErrorWithContext::with_message("Cannot initialize video subsystem", Box::new(x)))?,
        audio: sdl.audio()
            .map_err(|x| ErrorWithContext::with_message("Cannot initialize audio subsystem", Box::new(x)))?,
        event_pump: sdl.event_pump()
            .map_err(|x| ErrorWithContext::with_message("Cannot get SDL event pump", Box::new(x)))?,
        sdl
    });
    states::sdl::set(accessor);

    info!("SDL3 initialized");

    let window = Window::new(
        sdl_resource.get()
            .video
            .window("Game UwU", 1280, 720)
            .vulkan()
            .position_centered()
    ).map_err(|x| x.wrap("Failed to create window"))?;

    let (main_resource, accessor) = LocalResource::new("Main state", MainState {
        window
    });
    states::main::set(accessor);

    info!("Game window created");

    Ok(Resources {
        sdl_resource,
        main_resource
    })
}

async fn async_main(
    // Future is ready when a quit request received
    // from other places (other than SDL3) depends on
    // host system
    quit_request_receiver: impl Fn() -> Pin<Box<dyn Future<Output = ()>>>
) -> Result<(), ErrorWithContext<dyn Error + 'static>> {
    runtimes::init();
    let mut resources = init().await?;

    // Main game loop
    let mut do_quit = false;
    let target_frame_time = Duration::from_millis(1000 / 60);
    let mut prev_start_of_render = Instant::now();
    let mut start_of_render = Instant::now();

    let mut quit_future = quit_request_receiver().fuse();

    while !do_quit {
        if let Poll::Ready(_) = poll!(&mut quit_future) {
            // Quit trigger from somewhere else
            //do_quit = true;
        } else {
            handle_input(&mut resources, prev_start_of_render, start_of_render, &mut do_quit).await;
        }

        if do_quit {
            info!("Quit requested, quitting game");
        }

        let end_of_render = Instant::now();
        let render_time = end_of_render - start_of_render;
        
        // Lets poll for new request to access main resources
        // and wait till deadline passed
        let sleep_deadline = end_of_render + target_frame_time.saturating_sub(render_time);
        tokio::select! {
            _ = resources.poll_loop() => {}
            _ = tokio::time::sleep_until(sleep_deadline.into()) => {
                // Deadline passed, always sleep till deadline
                // as poll_loop will never be done
            }
        }

        prev_start_of_render = start_of_render;
        start_of_render = sleep_deadline;
    }

    Ok(())
}

async fn handle_input(resources: &mut Resources, prev_start_of_render: Instant, start_of_render: Instant, do_quit: &mut bool) {
    #[expect(unused)]
    let delta_time = start_of_render - prev_start_of_render;
    let main_window_id = resources.main_resource.get().window.get_id().get();
     
    // Render and input handle code here
    for event in resources.sdl_resource.get_mut().event_pump.poll_iter() {
        tokio::task::yield_now().await;
        match event {
            sdl3::event::Event::Quit { .. } => {
                *do_quit = true;
            }
            sdl3::event::Event::Window { window_id, win_event: sdl3::event::WindowEvent::CloseRequested, .. } => {
                if window_id != main_window_id {
                    continue;
                }
                *do_quit = true;
            }

            _ => {}
        }
    }
}





