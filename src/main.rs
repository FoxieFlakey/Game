#![feature(unsafe_cell_access)]
#![feature(current_thread_id)]

use std::error::Error;

use crate::{local_resource::LocalResource, util::ErrorWithContext, window::Window};

mod local_resource;
mod logging;
mod runtimes;
mod rendering;
mod util;
mod window;
mod states;

fn main() {
    logging::init();
    crate::info!("Hello, world!");

    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(x) => {
            runtimes::main::set(x);
            let res = runtimes::main::get().block_on(async_main());
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
  event: sdl3::EventSubsystem
}

struct ResourcesInitialized {
    sdl_resource: LocalResource<SdlState>
}

async fn init() -> Result<ResourcesInitialized, ErrorWithContext<dyn Error + 'static>> {
    rendering::init()?;

    let sdl = sdl3::init()
        .map_err(|x| ErrorWithContext::with_cause("Cannot initialize SDL3", Box::new(x)))?;
    let event = sdl.event()
        .map_err(|x| ErrorWithContext::with_cause("Cannot initialize events subsystem", Box::new(x)))?;
    let video = sdl.video()
        .map_err(|x| ErrorWithContext::with_cause("Cannot initialize video subsystem", Box::new(x)))?;
    let audio = sdl.audio()
        .map_err(|x| ErrorWithContext::with_cause("Cannot initialize audio subsystem", Box::new(x)))?;

    let (sdl_resource, accessor) = LocalResource::new("SDL subsystems", SdlState {
        sdl,
        event,
        video,
        audio
    });
    states::sdl::set(accessor);

    info!("SDL3 initialized");

    Ok(ResourcesInitialized {
        sdl_resource
    })
}

async fn async_main() -> Result<(), ErrorWithContext<dyn Error + 'static>> {
    runtimes::init();
    let resources = init().await?;
    let window = Window::new(
        resources.sdl_resource.get()
            .video
            .window("Game UwU", 1280, 720)
            .vulkan()
            .position_centered()
    ).map_err(|x| x.wrap("Failed to create window"))?;

    Ok(())
}







