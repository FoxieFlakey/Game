use smallvec::smallvec;

use crate::{events::EventHandleResult, screen::Screen};

pub struct LoadingScreen {
    
}

impl LoadingScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for LoadingScreen {
    fn handle_event(
        &mut self,
        _delta_time: std::time::Duration,
        _event: &sdl3::event::Event,
    ) -> anyhow::Result<crate::events::EventHandleResult>
    {
        Ok(EventHandleResult::Consumed)
    }
    
    fn render(
        &mut self,
        delta_time: std::time::Duration,
        output_view: &wgpu::TextureView,
        cmd_encoder_creator: &dyn Fn(&wgpu::CommandEncoderDescriptor) -> wgpu::CommandEncoder,
    ) -> anyhow::Result<smallvec::SmallVec<[wgpu::CommandBuffer; super::STACK_ALLOCATED_COUNT]>>
    {
        let mut encoder = cmd_encoder_creator(&wgpu::CommandEncoderDescriptor::default());
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    store: wgpu::StoreOp::Store,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        a: 1.0,
                        b: 0.0,
                        g: 0.0,
                        r: 0.0
                    })
                }
            })],
            ..Default::default()
        });
        
        // TODO: actually render a loading screen
        let _ = &mut render_pass;
        
        drop(render_pass);
        Ok(smallvec![ encoder.finish() ])
    }
}

