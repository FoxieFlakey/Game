#![feature(unsafe_cell_access)]
#![feature(current_thread_id)]
#![feature(min_specialization)]

use std::{
    error::Error,
    num::NonZero,
    pin::Pin,
    task::Poll,
    time::{Duration, Instant},
};

use futures::{FutureExt, poll};

use crate::{
    local_resource::LocalResource,
    rendering::Renderer,
    ui::UI,
    util::error::{CustomError, CustomErrorExt, Printable},
    window::Window,
};

mod events;
mod fail_safe;
mod local_resource;
mod logging;
mod rendering;
mod runtimes;
mod states;
mod ui;
mod util;
mod wgpu_async;
mod window;

fn main() {
    logging::init();
    if let Err(e) = fail_safe::init() {
        fatal!("Cannot initialize fail safe: {}", Printable(&e));
        return;
    }
    crate::info!("Hello, world!");

    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(x) => {
            runtimes::main::set(x);
            let res = runtimes::main::get()
                .block_on(fail_safe::fail_safe_guard(|x| Box::pin(async_main(x))));

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
    event_pump: sdl3::EventPump,
}

pub struct MainState {
    window: Window,
    ui: UI,
}

struct Resources {
    sdl_resource: LocalResource<SdlState>,
    main_resource: LocalResource<MainState>,
    renderer_resource: LocalResource<Renderer>,
}

impl Resources {
    async fn poll_loop(&mut self) -> ! {
        tokio::join!(
            self.sdl_resource.poll_loop(),
            self.main_resource.poll_loop(),
            self.renderer_resource.poll_loop(),
        );

        panic!("Something horribly gone wrong, poll loop for resources intended to never finishes");
    }
}

async fn init() -> Result<Resources, Box<CustomError<dyn Error + 'static>>> {
    rendering::init().await?;

    let sdl = sdl3::init().map_err(|x| x.context("Initializing SDL library"))?;
    let (sdl_resource, accessor) = LocalResource::new(
        "SDL subsystems",
        SdlState {
            event: sdl
                .event()
                .map_err(|x| x.context("Initializing event subsystem"))?,
            video: sdl
                .video()
                .map_err(|x| x.context("Initializing video subsystem"))?,
            audio: sdl
                .audio()
                .map_err(|x| x.context("Initializing audio subsystem"))?,
            event_pump: sdl
                .event_pump()
                .map_err(|x| x.context("Creating event pump"))?,
            sdl,
        },
    );
    states::sdl::set(accessor);

    info!("SDL3 initialized");

    let window = Window::new(
        sdl_resource
            .get()
            .video
            .window("Game UwU", 1280, 720)
            .vulkan()
            .position_centered(),
    )
    .map_err(|x| x.context("Creating main game window"))?;

    info!("Game window created");

    let gpu = window.with_surface(|x| rendering::gpu_lookup::find_gpu(x))?;
    let info = gpu.get_info();
    info!(
        "Using {} for rendering via {} at {}",
        info.name, info.backend, info.device_pci_bus_id
    );

    let mut renderer = Renderer::new(gpu).await?;
    let (width, height) = window.get_size();
    let default = NonZero::new(10).unwrap();
    renderer.set_output_size((
        NonZero::new(width).unwrap_or(default),
        NonZero::new(height).unwrap_or(default),
    ));
    info!("Initialized rendering engine");

    let (renderer_resource, accessor) = LocalResource::new("Rendering engine", renderer);
    states::renderer::set(accessor);

    let (main_resource, accessor) = LocalResource::new(
        "Main state",
        MainState {
            ui: UI::new(),
            window,
        },
    );
    states::main::set(accessor);

    Ok(Resources {
        sdl_resource,
        main_resource,
        renderer_resource,
    })
}

async fn async_main(
    // Future is ready when a quit request received
    // from other places (other than SDL3) depends on
    // host system
    quit_request_receiver: impl Fn() -> Pin<Box<dyn Future<Output = ()>>>,
) -> Result<(), Box<CustomError<dyn Error + 'static>>> {
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
            do_quit = true;
        } else {
            handle_input(
                &mut resources,
                prev_start_of_render,
                start_of_render,
                &mut do_quit,
            )
            .await?;

            do_render(&mut resources, prev_start_of_render, start_of_render).await?;
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

async fn handle_input(
    resources: &mut Resources,
    prev_start_of_render: Instant,
    start_of_render: Instant,
    do_quit: &mut bool,
) -> Result<(), Box<CustomError<dyn Error + 'static>>> {
    let delta_time = start_of_render - prev_start_of_render;
    let main_window_id = resources.main_resource.get().window.get_id().get();

    // Render and input handle code here
    for event in resources.sdl_resource.get_mut().event_pump.poll_iter() {
        tokio::task::yield_now().await;
        match event {
            sdl3::event::Event::Quit { .. } => {
                *do_quit = true;
            }
            sdl3::event::Event::Window {
                window_id,
                win_event: sdl3::event::WindowEvent::CloseRequested,
                ..
            } => {
                if window_id != main_window_id {
                    continue;
                }
                *do_quit = true;
            }

            _ => {
                resources
                    .main_resource
                    .get_mut()
                    .ui
                    .handle_input(delta_time, &event);
            }
        }
    }

    Ok(())
}

async fn do_render(
    resources: &mut Resources,
    prev_start_of_render: Instant,
    start_of_render: Instant,
) -> Result<(), Box<CustomError<dyn Error + 'static>>> {
    let delta_time = start_of_render - prev_start_of_render;
    let Some(permit) = resources
        .main_resource
        .get()
        .window
        .with_surface(|surface| resources.renderer_resource.get_mut().prep_render(surface))?
    else {
        return Ok(());
    };

    permit
        .render(|output, encoder| {
            resources
                .main_resource
                .get()
                .ui
                .render(delta_time, output, encoder);
        })
        .await;

    Ok(())
}
