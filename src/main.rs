#![feature(unsafe_cell_access)]
#![feature(current_thread_id)]
#![feature(min_specialization)]
#![feature(range_bounds_is_empty)]
#![feature(oneshot_channel)]

use std::{
    ffi::CStr,
    num::NonZero,
    pin::Pin,
    task::Poll,
    time::{Duration, Instant},
};

use anyhow::Context;
use futures::{FutureExt, future::OptionFuture, poll};
use glam::{Mat4, Quat, Vec3, Vec4};
use smallvec::SmallVec;

use crate::{
    local_resource::LocalResource,
    registries::Registries,
    rendering::Renderer,
    screen::{Screen, screen_stack::ScreenStack},
    util::static_gpu_buffer,
    window::Window,
};

mod events;
mod fail_safe;
mod local_resource;
mod logging;
mod registries;
mod registry;
mod rendering;
mod runtimes;
mod screen;
mod states;
mod ui;
mod util;
mod wgpu_async;
mod window;

fn main() {
    logging::init();
    if let Err(e) = fail_safe::init() {
        fatal!("Cannot initialize fail safe: {:?}", e);
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
                crate::fatal!("Cannot run game: {e:?}");
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
    screen_stack: ScreenStack,
}

struct Resources {
    sdl_resource: LocalResource<SdlState>,
    main_resource: LocalResource<MainState>,
    renderer_resource: LocalResource<Renderer>,
    registries_resource: LocalResource<Option<Registries>>,
}

impl Resources {
    async fn poll_loop(&mut self) -> ! {
        tokio::join!(
            self.sdl_resource.poll_loop(),
            self.main_resource.poll_loop(),
            self.renderer_resource.poll_loop(),
            self.registries_resource.poll_loop(),
        );

        panic!("Something horribly gone wrong, poll loop for resources intended to never finishes");
    }
}

async fn init() -> anyhow::Result<Resources> {
    rendering::init().await?;

    let sdl = sdl3::init().context("Initializing SDL library")?;

    info!("SDL3 initialized");
    info!("SDL3 version: {}", sdl3::version::version());
    let revision_string = sdl3::sys::version::SDL_GetRevision();
    if revision_string.is_null() {
        info!("SDL3 revision: unavailable");
    } else {
        let revision_string = unsafe { CStr::from_ptr(revision_string) }.to_string_lossy();
        info!("SDL3 revision: {revision_string}");
    }

    let (sdl_resource, accessor) = LocalResource::new(
        "SDL subsystems",
        SdlState {
            event: sdl.event().context("Initializing event subsystem")?,
            video: sdl.video().context("Initializing video subsystem")?,
            audio: sdl.audio().context("Initializing audio subsystem")?,
            event_pump: sdl.event_pump().context("Creating event pump")?,
            sdl,
        },
    );
    states::sdl::set(accessor);

    let window = Window::new(
        sdl_resource
            .get()
            .video
            .window("Game UwU", 1280, 720)
            .vulkan()
            .position_centered(),
    )
    .context("Creating main game window")?;

    info!("Game window created");

    let gpu = window
        .with_surface(|x| rendering::gpu_lookup::find_gpu(x))
        .context("Looking up compatible GPU")?;
    let info = gpu.get_info();
    info!(
        "Using {} for rendering via {} at {}",
        info.name, info.backend, info.device_pci_bus_id
    );

    let mut renderer = Renderer::new(gpu).await.context("Initializing renderer")?;
    let (width, height) = window.get_size();
    let default = NonZero::new(10).unwrap();
    renderer.set_output_size((
        NonZero::new(width).unwrap_or(default),
        NonZero::new(height).unwrap_or(default),
    ));
    info!("Initialized rendering engine");

    // This is necessary, so renderer can fetch additional
    // information that cannot be fetched without knowing
    // the target surface (which has special limitation)
    window.with_surface(|x| renderer.configure_surface(x));

    states::main_dev::set(renderer.get_device().clone());
    states::data_loader::set(renderer.data_loader());
    states::surface_format::set(renderer.get_output_format());
    let (renderer_resource, accessor) = LocalResource::new("Rendering engine", renderer);
    states::renderer::set(accessor);

    let (main_resource, accessor) = LocalResource::new(
        "Main state",
        MainState {
            screen_stack: ScreenStack::new(),
            window,
        },
    );
    states::main::set(accessor);

    let (registries_resource, accessor) = LocalResource::new("Game registries", None);
    states::registries::set(accessor);

    Ok(Resources {
        sdl_resource,
        main_resource,
        renderer_resource,
        registries_resource,
    })
}

async fn late_init() -> anyhow::Result<impl FnOnce(&mut Resources) -> anyhow::Result<()>> {
    info!("Late initializing UI");
    ui::init().context("Initializing UI")?;

    info!("Late initializing registry");
    let registries = registries::load_registries()
        .await
        .context("Initializing registries")?;

    Ok(move |resources: &mut Resources| {
        let mut regs = resources.registries_resource.get_mut();
        if regs.is_some() {
            return Err(anyhow::anyhow!(
                "Something already initialized the registries?!"
            ));
        }

        *regs = Some(registries);

        struct Rect;
        
        impl screen::Screen for Rect {
            fn handle_event(
                &mut self,
                _delta_time: Duration,
                _event: &sdl3::event::Event,
            ) -> anyhow::Result<events::EventHandleResult>
            {
                Ok(events::EventHandleResult::Pass)
            }
            
            fn render(
                &mut self,
                _delta_time: Duration,
                output_view: &wgpu::TextureView,
                cmd_encoder_creator: &dyn Fn(&wgpu::CommandEncoderDescriptor) -> wgpu::CommandEncoder,
            ) -> anyhow::Result<SmallVec<[wgpu::CommandBuffer; screen::STACK_ALLOCATED_COUNT]>>
            {
                static_gpu_buffer!(
                    static Vertex RECTANGLES: LazyLock<VecBuf<[ui::primitives::ColoredRectangle]>> => [
                        ui::primitives::ColoredRectangle {
                            color: Vec4::new(0.5, 0.2, 0.2, 1.0),
                            transform: Mat4::from_scale_rotation_translation(
                                Vec3::new(0.2, 0.5, 1.0),
                                Quat::from_rotation_z(30.0_f32.to_radians()),
                                Vec3::new(0.0, -0.4, 0.0)
                            )
                        },

                        ui::primitives::ColoredRectangle {
                            color: Vec4::new(0.0, 0.6, 0.2, 1.0),
                            transform: Mat4::from_scale_rotation_translation(
                                Vec3::new(0.2, 0.1, 1.0),
                                Quat::from_rotation_z(50.0_f32.to_radians()),
                                Vec3::new(0.0, 0.4, 0.0)
                            )
                        }
                    ];
                );

                let mut cmd_buf = cmd_encoder_creator(&wgpu::CommandEncoderDescriptor {
                    label: Some("Draw colored rectangles"),
                });

                let mut render_pass = cmd_buf.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Draw colored rectangle"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: output_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });

                ui::primitives::render_colored_rectangle(&mut render_pass, &RECTANGLES);

                drop(render_pass);
                Ok(smallvec::smallvec![ cmd_buf.finish() ])
            }
        }

        // Lets display the glory rectangles :>
        resources.main_resource.get_mut()
            .screen_stack
            .push_screen(Rect);
        
        Ok(())
    })
}

async fn async_main(
    // Future is ready when a quit request received
    // from other places (other than SDL3) depends on
    // host system
    quit_request_receiver: impl Fn() -> Pin<Box<dyn Future<Output = ()>>>,
) -> anyhow::Result<()> {
    runtimes::init();
    let mut resources = init()
        .await
        .context("Initializing minimal game subsystems")?;
    info!("Minimal game subsystems ready, initializing other resources on background");

    let a = runtimes::background::spawn(late_init());
    let mut registry_init_handle = OptionFuture::from(Some(a));

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
            .context("Handling events")?;

            do_render(&mut resources, start_of_render - prev_start_of_render)
                .context("Rendering game")?;
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

            Some(result) = &mut registry_init_handle => {
                registry_init_handle = OptionFuture::from(None);

                match result {
                    Ok(Ok(on_main_thread)) => {
                        on_main_thread(&mut resources)
                            .inspect_err(|x| {
                                fatal!("Cannot do late init on main thread initialization: {x}");
                            })
                            .context("Executing the late init on main thread")?;
                        info!("Game initialization completed!");
                    }

                    Ok(Err(mut e)) => {
                        e = e.context("Initializing rest of game");
                        fatal!("Game initialization failed: {e:#}");
                        return Err(e);
                    }

                    Err(e) => {
                        fatal!("Cannot wait for game initialization task: {e:#}");
                        return Err(anyhow::Error::new(e).context("Waiting the game initialization task"));
                    }
                }
            }
        }

        prev_start_of_render = start_of_render;
        start_of_render = sleep_deadline;
    }

    Ok(())
}

fn handle_input(
    resources: &mut Resources,
    prev_start_of_render: Instant,
    start_of_render: Instant,
    do_quit: &mut bool,
) -> anyhow::Result<()> {
    let delta_time = start_of_render - prev_start_of_render;
    let main_window_id = resources.main_resource.get().window.get_id().get();

    // Render and input handle code here
    for event in resources.sdl_resource.get_mut().event_pump.poll_iter() {
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
                    .screen_stack
                    .handle_event(delta_time, &event)?;
            }
        }
    }

    Ok(())
}

fn do_render(resources: &mut Resources, delta_time: Duration) -> anyhow::Result<()> {
    let mut renderer = resources.renderer_resource.get_mut();
    let Some(permit) = resources
        .main_resource
        .get()
        .window
        .with_surface(|surface| renderer.prep_render(surface))?
    else {
        return Ok(());
    };

    permit.render(|output, encoder_maker| {
        resources
            .main_resource
            .get_mut()
            .screen_stack
            .render(delta_time, output, encoder_maker)
            .map(|cmds| ((), cmds))
    })?;

    Ok(())
}
