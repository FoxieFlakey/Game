use std::{error::Error, time::Duration};

use smallvec::{SmallVec, smallvec};

use crate::{events::EventHandleResult, rendering, util::error::CustomError};

pub mod component;

pub struct UI {}

impl UI {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_input(
        &mut self,
        delta_time: Duration,
        event: &sdl3::event::Event,
    ) -> EventHandleResult {
        EventHandleResult::Consumed
    }

    pub fn render(
        &self,
        delta_time: Duration,
        output: &wgpu::TextureView,
        encoder_maker: &dyn Fn(&wgpu::CommandEncoderDescriptor) -> wgpu::CommandEncoder,
    ) -> Result<
        (
            (),
            SmallVec<[wgpu::CommandBuffer; rendering::STACK_ALLOCATED_COUNT_OF_BUFS]>,
        ),
        Box<CustomError<dyn Error>>,
    > {
        Ok(((), smallvec![]))
    }
}
