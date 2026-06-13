// A stack of screen, where multiple possibly
// transparent screen stacks
//
// This also handles setting up proper stuffs to
// allow that

use std::{error::Error, time::Duration};

use smallvec::SmallVec;

use crate::{
    events::EventHandleResult,
    screen::{STACK_ALLOCATED_COUNT, Screen},
    util::error::CustomError,
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
    ) -> Result<EventHandleResult, Box<CustomError<dyn Error + 'static>>> {
        // Top most screen receives events first beffore lower level
        for screen in self.stack.iter_mut().rev() {
            screen.handle_event(delta_time, &event)?;
        }
        Ok(EventHandleResult::Pass)
    }

    fn render(
        &mut self,
        delta_time: Duration,
        output_view: &wgpu::TextureView,
        cmd_encoder_creator: &dyn Fn(&wgpu::CommandEncoderDescriptor) -> wgpu::CommandEncoder,
    ) -> Result<
        SmallVec<[wgpu::CommandBuffer; STACK_ALLOCATED_COUNT]>,
        Box<CustomError<dyn Error + 'static>>,
    > {
        let mut result = SmallVec::new();

        // Bottom most screen draw first, then top
        for screen in self.stack.iter_mut().rev() {
            result.append(&mut screen.render(delta_time, output_view, cmd_encoder_creator)?);
        }

        Ok(result)
    }
}
