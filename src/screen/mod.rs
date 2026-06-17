use std::time::Duration;

use smallvec::SmallVec;

use crate::events::EventHandleResult;

pub mod loading_screen;
pub mod screen_stack;

pub use loading_screen::LoadingScreen;

pub const STACK_ALLOCATED_COUNT: usize = 8;

pub trait Screen: 'static {
    fn render(
        &mut self,
        delta_time: Duration,
        output_view: &wgpu::TextureView,
        cmd_encoder_creator: &dyn Fn(&wgpu::CommandEncoderDescriptor) -> wgpu::CommandEncoder,
    ) -> anyhow::Result<SmallVec<[wgpu::CommandBuffer; STACK_ALLOCATED_COUNT]>>;

    fn handle_event(
        &mut self,
        delta_time: Duration,
        event: &sdl3::event::Event,
    ) -> anyhow::Result<EventHandleResult>;
}
