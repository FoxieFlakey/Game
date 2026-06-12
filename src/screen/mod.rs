use std::error::Error;

use smallvec::SmallVec;

use crate::{events::EventHandleResult, util::error::CustomError};

pub mod screen_stack;

pub const STACK_ALLOCATED_COUNT: usize = 8;

pub trait Screen: 'static {
    fn render(
        &mut self,
        output_view: &wgpu::TextureView,
        cmd_encoder_creator: &dyn Fn() -> wgpu::CommandEncoder,
    ) -> Result<SmallVec<[wgpu::CommandBuffer; STACK_ALLOCATED_COUNT]>, Box<CustomError<dyn Error>>>;

    fn handle_events(
        &mut self,
        event: &sdl3::event::Event,
    ) -> Result<EventHandleResult, Box<CustomError<dyn Error>>>;
}
