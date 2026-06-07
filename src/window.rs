use std::num::NonZero;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::{
    rendering,
    util::error::{CustomError, CustomErrorExt},
};

pub struct Window {
    // This has to be dropped FIRST!
    // its actually not 'static and has to live
    // shorter than sdl3::Window it came from
    surface: wgpu::Surface<'static>,
    win: sdl3::video::Window,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateWindowError {
    #[error("Create window with SDL3 failed: {0}")]
    Create(sdl3::video::WindowBuildError),
    #[error("Creating wgpu surface failed: {0}")]
    CreateWgpuSurface(wgpu::CreateSurfaceError),
    #[error("Cant retrieve window handle: {0}")]
    GetWindowHandle(raw_window_handle::HandleError),
    #[error("Cant retrieve display handle: {0}")]
    GetDisplayHandle(raw_window_handle::HandleError),
}

impl Window {
    pub fn new(
        builder: &sdl3::video::WindowBuilder,
    ) -> Result<Self, CustomError<CreateWindowError>> {
        let win = builder.build()
            .map_err(|e| CreateWindowError::Create(e).into_custom_err())?;

        Ok(Window {
            surface: unsafe {
                rendering::WGPU
                    .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                        // TODO: Handle this properly so platform which dont have display_handle
                        // dont need this to be Some
                        raw_display_handle: Some(
                            win.display_handle()
                                .map_err(|e| {
                                    CreateWindowError::GetDisplayHandle(e)
                                        .into_custom_err()
                                })?
                                .as_raw(),
                        ),
                        raw_window_handle: win
                            .window_handle()
                            .map_err(|e| {
                                CreateWindowError::GetWindowHandle(e)
                                    .into_custom_err()
                            })?
                            .as_raw(),
                    })
                    .map_err(|e| {
                        CreateWindowError::CreateWgpuSurface(e)
                            .into_custom_err()
                    })?
            },
            win,
        })
    }

    pub fn get_id(&self) -> NonZero<u32> {
        let ret = self.win.id();
        if ret == 0 {
            // This function is consired "important" so it shouldnt fail, if it fails we panic
            panic!(
                "Cannot get window ID, should not happen: {}",
                sdl3::get_error()
            );
        }
        NonZero::new(ret).unwrap()
    }

    pub fn get_size(&self) -> (u32, u32) {
        // SDL3 does not provide the return value for SDL_GetWindowSize, so Foxie has to call it
        // directly

        let mut w: std::ffi::c_int = 0;
        let mut h: std::ffi::c_int = 0;

        // SAFETY: Provided valid pointer to w and h
        let ret = unsafe { sdl3::sys::video::SDL_GetWindowSize(self.win.raw(), &mut w, &mut h) };
        if ret == false {
            // This function is consired "important" so it shouldnt fail, if it fails we panic
            panic!(
                "Cannot retrieve window size, should not happen: {}",
                sdl3::get_error()
            );
        }
        (w as u32, h as u32)
    }

    pub fn set_visibility(
        &mut self,
        is_visible: bool,
    ) -> Result<(), CustomError<sdl3::Error>> {
        let result;
        if is_visible {
            result = self.win.show();
        } else {
            result = self.win.hide();
        }

        if !result {
            Err(sdl3::get_error().into_custom_err())
        } else {
            Ok(())
        }
    }

    pub fn with_surface<R, F: FnOnce(&wgpu::Surface<'_>) -> R>(&self, func: F) -> R {
        func(&self.surface)
    }
}
