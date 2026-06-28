// A stack of screen, where multiple possibly
// transparent screen stacks
//
// This also handles setting up proper stuffs to
// allow that

use std::time::Duration;

use smallvec::SmallVec;

use crate::{
    events::EventHandleResult,
    screen::{STACK_ALLOCATED_COUNT, Screen},
};

pub struct ScreenStack {
    stack: Vec<Box<dyn Screen>>,
}

impl ScreenStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push_screen<T: Screen>(&mut self, screen: T) {
        self.stack.push(Box::new(screen) as Box<dyn Screen>);
    }

    pub fn pop_screen(&mut self) -> Option<Box<dyn Screen>> {
        self.stack.pop()
    }
}

impl Screen for ScreenStack {
    fn handle_event(
        &mut self,
        delta_time: Duration,
        event: &sdl3::event::Event,
    ) -> anyhow::Result<EventHandleResult> {
        // Top most screen receives events first beffore lower level
        for screen in self.stack.iter_mut().rev() {
            screen.handle_event(delta_time, &event)?;
        }
        Ok(EventHandleResult::Pass)
    }

    fn on_resize(&mut self, new_screen_width: f32, new_screen_height: f32) {
        // Top most screen receives resizing first beffore lower level
        for screen in self.stack.iter_mut().rev() {
            screen.on_resize(new_screen_width, new_screen_height);
        }
    }

    fn render(
        &mut self,
        delta_time: Duration,
        output_view: &wgpu::TextureView,
        cmd_encoder_creator: &dyn Fn(&wgpu::CommandEncoderDescriptor) -> wgpu::CommandEncoder,
    ) -> anyhow::Result<SmallVec<[wgpu::CommandBuffer; STACK_ALLOCATED_COUNT]>> {
        let mut result = SmallVec::new();

        // Bottom most screen draw first, then top
        for screen in self.stack.iter_mut() {
            result.append(&mut screen.render(delta_time, output_view, cmd_encoder_creator)?);
        }

        Ok(result)
    }
}
