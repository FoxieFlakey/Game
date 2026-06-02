#![feature(unsafe_cell_access)]
#![feature(current_thread_id)]

use std::{error::Error, pin::Pin, task::Poll, time::{Duration, Instant}};

use futures::{FutureExt, poll};

use crate::{local_resource::LocalResource, util::ErrorWithContext, window::Window};

mod local_resource;
mod logging;
mod runtimes;
mod rendering;
mod util;
mod window;
mod states;
mod fail_safe;

fn main() {
    logging::init();
    if let Err(e) = fail_safe::init() {
        fatal!("Cannot initialize fail safe: {e}");
        return;
    }
    crate::info!("Hello, world!");
    
    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(x) => {
            runtimes::main::set(x);
            let res = runtimes::main::get().block_on(fail_safe::fail_safe_guard(|x| {
                Box::pin(async_main(x))
            }));

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





