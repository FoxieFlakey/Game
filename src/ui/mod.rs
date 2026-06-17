use std::time::Duration;

use smallvec::{SmallVec, smallvec};

use crate::{events::EventHandleResult, screen::Screen};

pub mod component;
pub mod primitives;

pub struct UI {}

impl UI {
    pub fn new() -> Self {
        Self {}
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
        Ok(smallvec![])
    }
}
