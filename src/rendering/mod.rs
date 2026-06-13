use std::{
    cmp,
    collections::VecDeque,
    num::{NonZero, NonZeroU32},
    sync::LazyLock,
};

use anyhow::anyhow;
use smallvec::SmallVec;

use crate::rendering::data_loader::DataLoader;

pub mod buffer;
pub mod data_loader;
pub mod gpu_lookup;
pub mod util;

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
pub async fn init() -> anyhow::Result<()> {
    LazyLock::force(&WGPU);
    gpu_lookup::init().await?;
    Ok(())
}

pub struct Renderer {
    queue: wgpu::Queue,
    device: wgpu::Device,
    gpu: wgpu::Adapter,
    config: Option<wgpu::SurfaceConfiguration>,
    need_configure: bool,
    output_size: (NonZeroU32, NonZeroU32),

    // per frame data for render aheads
    // where GPU working on N frame while
    // CPU prepares N+1, N+2, etc frame
    //
    // This will be empty, till surface is
    // configured
    per_frame_data_cache: VecDeque<Frame>,
    inflight_frames: VecDeque<InFlightFrame>,
    inflight_max_count: usize,

    device_poller: util::DevicePoller,
}

struct InFlightFrame {
    poller: Option<util::SubmissionPoller>,
    frame_data: Frame,
}

struct Frame {
    output: wgpu::Texture,
}

#[derive(thiserror::Error, Debug)]
pub enum RendererCreateFailed {
    #[error("Cannot request device: {0}")]
    CannotRequestDevice(#[from] wgpu::RequestDeviceError),
}

pub struct RenderPermit<'a> {
    renderer: &'a mut Renderer,

    // A single Option to blit to surface
    // if its None, then blitting to surface can't
    // happen. If in single render in RenderPermit
    // found two frames ready
    output_surface: Option<wgpu::SurfaceTexture>,
}

impl Renderer {
    pub async fn new(gpu: &wgpu::Adapter) -> anyhow::Result<Self> {
        let desc = wgpu::DeviceDescriptor {
            ..Default::default()
        };

        let (device, queue) = gpu
            .request_device(&desc)
            .await?;

        Ok(Self {
            device_poller: util::DevicePoller::new(device.clone()),
            device,
            queue,
            gpu: gpu.clone(),
            config: None,
            need_configure: true,
            output_size: (NonZero::new(10).unwrap(), NonZero::new(10).unwrap()),
            inflight_frames: VecDeque::new(),
            per_frame_data_cache: VecDeque::new(),
            inflight_max_count: 1,
        })
    }

    pub fn set_output_size(&mut self, size: (NonZeroU32, NonZeroU32)) {
        self.output_size = size;
        self.need_configure = true;
    }

    fn configure_surface(&mut self, surface: &wgpu::Surface<'_>) {
        // Surface hasn't been configured
        let device = &self.device;
        let gpu = &self.gpu;
        let caps = surface.get_capabilities(gpu);
        let format = caps
            .formats
            .iter()
            .filter(|x| x.is_srgb())
            .next()
            .copied()
            .unwrap_or_else(|| {
                crate::warn!(
                    "cannot find optimal display format using suboptimal {:?} format",
                    caps.formats[0]
                );
                caps.formats[0]
            });

        let new_config = wgpu::SurfaceConfiguration {
            format,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            desired_maximum_frame_latency: 2,
            width: self.output_size.0.get(),
            height: self.output_size.1.get(),
            usage: wgpu::TextureUsages::COPY_DST,
            view_formats: vec![],
        };

        surface.configure(device, &new_config);
        self.update_frame_data_caches(&new_config);
        self.config = Some(new_config);
    }

    fn update_frame_data_caches(&mut self, new_surface_config: &wgpu::SurfaceConfiguration) {
        // Clearing the entire inflight queue (dropping previous frames)
        // and erasing cache
        self.inflight_frames.clear();
        self.per_frame_data_cache.clear();

        for _ in 0..self.inflight_max_count {
            // Recreate "staging" texture where GPU renders into
            // before finally blitted to surface
            self.per_frame_data_cache.push_back(Frame {
                output: self.device.create_texture(&wgpu::TextureDescriptor {
                    dimension: wgpu::TextureDimension::D2,
                    label: Some("Intemediary texture"),
                    mip_level_count: 1,
                    sample_count: 1,
                    view_formats: &[],
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    size: wgpu::Extent3d {
                        depth_or_array_layers: 1,
                        height: new_surface_config.height,
                        width: new_surface_config.width,
                    },
                    format: new_surface_config.format,
                }),
            });
        }
    }

    pub fn data_loader(&self) -> DataLoader {
        DataLoader::new(self.device.clone(), self.queue.clone())
    }

    pub fn prep_render<'a>(
        &'a mut self,
        surface: &wgpu::Surface<'_>,
    ) -> anyhow::Result<Option<RenderPermit<'a>>> {
        if self.need_configure {
            self.configure_surface(surface);
            self.need_configure = false;
        }

        let output_surface;
        match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Lost => {
                return Err(anyhow!("Device is lost!"));
            }

            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(anyhow!("Validation error occured!"));
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

            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(None);
            }
        }

        Ok(Some(RenderPermit {
            renderer: self,
            output_surface: Some(output_surface),
        }))
    }

    pub fn get_device(&self) -> &wgpu::Device {
        &self.device
    }
}

pub const STACK_ALLOCATED_COUNT_OF_BUFS: usize = 8;

impl RenderPermit<'_> {
    // If the frame can be presented to surface, it returns true. else false
    #[must_use = "if not checked, necessary requeuing frame to be present again in future. Causing some frames to be lost"]
    fn present_frame_to_surface(&mut self, frame: &Frame) -> bool {
        let Some(output_surface) = self.output_surface.take() else {
            return false;
        };

        let dest = &output_surface.texture;
        let src = &frame.output;

        let mut cmd =
            self.renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Blit final frame to display"),
                });

        cmd.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: src,
                mip_level: 0,
                aspect: wgpu::TextureAspect::All,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::TexelCopyTextureInfo {
                texture: dest,
                mip_level: 0,
                aspect: wgpu::TextureAspect::All,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::Extent3d {
                depth_or_array_layers: 1,
                height: cmp::min(dest.height(), src.height()),
                width: cmp::min(dest.width(), src.width()),
            },
        );

        let id = self.renderer.queue.submit([cmd.finish()]);

        // Have to wait, or else if CPU fast enough it would reach the this function again before the
        // Texture for surface is done being used by GPU (to be blitted into)
        util::wait_device(&self.renderer.device, id);

        // Now surface contains the rendered surface
        output_surface.present();

        true
    }

    // Return bool if any of inflight is presented
    fn check_inflight(&mut self) {
        let Some(mut oldest_inflight) = self.renderer.inflight_frames.pop_front() else {
            return;
        };

        let is_completed;
        if let Some(poller) = oldest_inflight.poller.as_mut() {
            is_completed = poller.poll();
        } else {
            // Because poller don't exists, then its completed
            is_completed = true;
        }

        // If an inflight frme completed, then copy/present to surface
        if is_completed {
            oldest_inflight.poller = None;

            // The inflight frame is ready, it can be blitted/copied to final surface
            if self.present_frame_to_surface(&oldest_inflight.frame_data) {
                self.renderer
                    .per_frame_data_cache
                    .push_back(oldest_inflight.frame_data);
            } else {
                // Couldn't blit to surface because surface is already presented
                // push back to queue
                self.renderer.inflight_frames.push_front(oldest_inflight);
            }
        } else {
            // Not ready, put it back in front
            self.renderer.inflight_frames.push_front(oldest_inflight);
        }
    }

    fn wait_inflight(&mut self) -> Option<Frame> {
        let mut oldest_inflight = self
            .renderer
            .inflight_frames
            .pop_front()
            .expect("If cache of frame data empty, there must be atleast one inflight");

        if let Some(poller) = oldest_inflight.poller.take() {
            // If GPU still doing stuff, lets wait
            poller.wait();
        }

        // Then the frame can be blitted/copied to final surface
        if self.present_frame_to_surface(&oldest_inflight.frame_data) {
            Some(oldest_inflight.frame_data)
        } else {
            // Present didn't happen, lets requeue it to attempt present again
            // in future
            self.renderer.inflight_frames.push_front(oldest_inflight);
            None
        }
    }

    pub fn render<F, T, E>(mut self, render_code: F) -> Result<T, E>
    where
        F: FnOnce(
            &wgpu::TextureView,
            &dyn Fn(&wgpu::CommandEncoderDescriptor) -> wgpu::CommandEncoder,
        ) -> Result<(T, SmallVec<[wgpu::CommandBuffer; 8]>), E>,
    {
        let frame;

        if let Some(data) = self.renderer.per_frame_data_cache.pop_front() {
            // There available frame data, use it
            frame = data;

            // The surface hasn't been presented, lets check if there any inflight that is done
            self.check_inflight();
        } else {
            frame = self
                .wait_inflight()
                .expect("cannot happen, the surface havent beeen presented.");
        }

        let output = frame
            .output
            .create_view(&wgpu::TextureViewDescriptor::default());
        let clear_command = clear_background(&output, &self.renderer.device);

        let mut cmds = SmallVec::<[wgpu::CommandBuffer; 32]>::new();
        cmds.push(clear_command);

        let (ret, mut render_cmds) = render_code(&output, &|desc| {
            self.renderer.device.create_command_encoder(desc)
        })?;
        cmds.append(&mut render_cmds);

        let id = self.renderer.queue.submit(cmds);
        self.renderer.inflight_frames.push_back(InFlightFrame {
            poller: Some(self.renderer.device_poller.create_poll(id)),
            frame_data: frame,
        });
        Ok(ret)
    }
}

fn clear_background(output: &wgpu::TextureView, device: &wgpu::Device) -> wgpu::CommandBuffer {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Clear background encoder"),
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
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    });

    encoder.finish()
}
