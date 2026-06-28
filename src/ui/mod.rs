use std::time::Duration;

use glam::Mat4;
use smallvec::{SmallVec, smallvec};

use crate::{
    events::EventHandleResult, rendering::buffer::VecBuf, screen::Screen, states, util::vec_buf2,
};

pub mod component;
pub mod primitives;

pub struct UI {
    screen_width: f32,
    screen_height: f32,
    camera: VecBuf<Camera>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Camera {
    projection_matrix: Mat4,
}

impl UI {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let mut this = Self {
            screen_width,
            screen_height,
            camera: vec_buf2!(
                Uniform,
                [Camera {
                    projection_matrix: Mat4::IDENTITY
                }]
            ),
        };

        this.on_resize(screen_width, screen_height);
        this
    }
}

pub fn init() -> anyhow::Result<()> {
    primitives::init();

    Ok(())
}

impl Screen for UI {
    fn handle_event(
        &mut self,
        delta_time: Duration,
        event: &sdl3::event::Event,
    ) -> anyhow::Result<EventHandleResult> {
        Ok(EventHandleResult::Pass)
    }

    fn render(
        &mut self,
        delta_time: Duration,
        output_view: &wgpu::TextureView,
        cmd_encoder_creator: &dyn Fn(&wgpu::CommandEncoderDescriptor) -> wgpu::CommandEncoder,
    ) -> anyhow::Result<SmallVec<[wgpu::CommandBuffer; crate::screen::STACK_ALLOCATED_COUNT]>> {
        let mut encoder = cmd_encoder_creator(&wgpu::CommandEncoderDescriptor::default());
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                depth_slice: None,
                view: output_view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                resolve_target: None,
            })],
            ..Default::default()
        });

        // TODO: Render the components here

        drop(render_pass);
        Ok(smallvec![encoder.finish()])
    }

    fn on_resize(&mut self, new_screen_width: f32, new_screen_height: f32) {
        self.screen_width = new_screen_width;
        self.screen_height = new_screen_height;

        self.camera.set(
            0,
            states::data_loader::get(),
            &Camera {
                projection_matrix: Mat4::orthographic_lh(
                    0.0,
                    new_screen_width,
                    0.0,
                    new_screen_height,
                    0.0,
                    100000.0,
                ),
            },
        );
    }
}
