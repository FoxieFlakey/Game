use std::{error::Error, time::Duration};

use smallvec::{SmallVec, smallvec};

use crate::{events::EventHandleResult, screen::Screen, util::error::CustomError};

pub mod component;

pub struct UI {}

impl UI {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for UI {
    fn handle_event(
        &mut self,
        delta_time: Duration,
        event: &sdl3::event::Event,
    ) -> Result<EventHandleResult, Box<CustomError<dyn Error + 'static>>> {
        Ok(EventHandleResult::Pass)
    }

    fn render(
        &mut self,
        delta_time: Duration,
        output_view: &wgpu::TextureView,
        cmd_encoder_creator: &dyn Fn(&wgpu::CommandEncoderDescriptor) -> wgpu::CommandEncoder,
    ) -> Result<
        SmallVec<[wgpu::CommandBuffer; crate::screen::STACK_ALLOCATED_COUNT]>,
        Box<CustomError<dyn Error + 'static>>,
    > {
        Ok(smallvec![])
    }
}
