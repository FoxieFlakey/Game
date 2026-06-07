use std::{error::Error, num::{NonZero, NonZeroU32}, sync::LazyLock};

use thiserror::Error;

use crate::{
    util::{StringError, error::{CustomError, CustomErrorExt}},
    wgpu_async,
};

pub mod gpu_lookup;

pub static WGPU: LazyLock<wgpu::Instance> = LazyLock::new(|| {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        flags: wgpu::InstanceFlags::debugging(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds {
            for_resource_creation: None,
            for_device_loss: None,
        },
        backend_options: wgpu::BackendOptions::default(),
        display: None,
    })
});

// Init global stuffs about rendering
// that is not per renderer
pub async fn init() -> Result<(), Box<CustomError<dyn Error + 'static>>> {
    LazyLock::force(&WGPU);
    gpu_lookup::init().await?;
    Ok(())
}

pub struct Renderer {
    queue: wgpu_async::AsyncQueue,
    gpu: wgpu::Adapter,
    config: Option<wgpu::SurfaceConfiguration>,
    need_configure: bool,
    output_size: (NonZeroU32, NonZeroU32)
}

#[derive(Error, Debug)]
pub enum RendererCreateFailed {
    #[error("Cannot request device: {0}")]
    CannotRequestDevice(#[from] wgpu::RequestDeviceError),
}

pub struct RenderPermit<'a> {
    renderer: &'a mut Renderer,
    output_surface: wgpu::SurfaceTexture
}

impl Renderer {
    pub async fn new(gpu: &wgpu::Adapter) -> Result<Self, CustomError<RendererCreateFailed>> {
        let desc = wgpu::DeviceDescriptor {
            ..Default::default()
        };

        let (device, queue) = gpu
            .request_device(&desc)
            .await
            .map_err(|x| x.into_custom_err())
            .map_err(CustomError::convert)?;

        Ok(Self {
            queue: wgpu_async::AsyncQueue::new(device, queue),
            gpu: gpu.clone(),
            config: None,
            need_configure: true,
            output_size: (NonZero::new(10).unwrap(), NonZero::new(10).unwrap())
        })
    }
    
    pub fn set_output_size(&mut self, size: (NonZeroU32, NonZeroU32)) {
        self.output_size = size;
        self.need_configure = true;
    }
    
    fn configure_surface(&mut self, surface: &wgpu::Surface<'_>) {
        // Surface hasn't been configured
        let device = self.queue.get_device();
        let gpu = &self.gpu;
        let caps = surface.get_capabilities(gpu);
        let format = caps.formats
            .iter()
            .filter(|x| x.is_srgb())
            .next()
            .copied()
            .unwrap_or_else(|| {
                crate::warn!("cannot find optimal display format using suboptimal {:?} format", caps.formats[0]);
                caps.formats[0]
            });
        
        let new_config = wgpu::SurfaceConfiguration {
            format,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            desired_maximum_frame_latency: 2,
            width: self.output_size.0.get(),
            height: self.output_size.1.get(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: vec![]
        };
        
        surface.configure(device, &new_config);
        self.config = Some(new_config);
    }
    
    pub fn prep_render<'a>(&'a mut self, surface: &wgpu::Surface<'_>) -> Result<Option<RenderPermit<'a>>, CustomError<StringError>> {
        if self.need_configure {
            self.configure_surface(surface);
            self.need_configure = false;
        }
        
        let output_surface;
        match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Lost => {
                return Err(StringError::new("Device is lost!").into_custom_err())
            }
            
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(StringError::new("Validation error occured!").into_custom_err())
            }
            
            wgpu::CurrentSurfaceTexture::Success(texture) => {
                output_surface = texture;
            }
            
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                self.need_configure = true;
                output_surface = texture;
            }
            
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface(surface);
                return Ok(None);
            }
            
            wgpu::CurrentSurfaceTexture::Timeout |
            wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(None)
            }
        }
        
        Ok(Some(RenderPermit {
            renderer: self,
            output_surface
        }))
    }
}

impl RenderPermit<'_> {
    pub async fn render<F, R>(self, render_code: F) -> R
        where F: FnOnce(&wgpu::TextureView, &mut wgpu::CommandEncoder) -> R,
    {
        let output = self.output_surface.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let clear_command = clear_background(&output, &self.renderer.queue);
        
        let mut encoder = self.renderer.queue.get_device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Main render code")
        });
        
        let ret = render_code(&output, &mut encoder);
        self.renderer.queue.submit([
            clear_command,
            encoder.finish()
        ]).await;
        self.output_surface.present();
        ret
    }
}

fn clear_background(output: &wgpu::TextureView, queue: &wgpu_async::AsyncQueue) -> wgpu::CommandBuffer {
    let mut encoder = queue.get_device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Clear background encoder")
    });
    
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Clear background"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: output,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.97,
                    g: 0.63,
                    b: 0.27,
                    a: 1.0
                }),
                store: wgpu::StoreOp::Store
            }
        })],
        ..Default::default()
    });
    
    encoder.finish()
}

