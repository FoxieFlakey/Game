use std::time::Duration;

use crate::events::EventHandleResult;

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
        encoder: &wgpu::CommandEncoder,
    ) {
    }
}
