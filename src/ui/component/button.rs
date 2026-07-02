use std::cell::Cell;

use crate::{
    events::{Event, EventHandleResult},
    ui::{
        BuilderContext,
        component::{Children, ComponentBuilder, ComponentTrait},
    },
    util::{impl_const_default, taffy_style},
};

pub struct Button {
    on_click: Box<dyn FnMut()>,
    is_down: bool,
}

impl ComponentTrait for Button {
    fn handle_event(
        &mut self,
        _transform_matrix: glam::Mat4,
        width: f32,
        height: f32,
        _delta_time: std::time::Duration,
        event: crate::events::Event,
    ) -> crate::events::EventHandleResult {
        match event {
            Event::MouseDown { x, y, .. } => {
                if (0.0..width).contains(&x) && (0.0..height).contains(&y) {
                    self.is_down = true;
                }
            }

            Event::MouseUp { x, y, .. } => {
                if (0.0..width).contains(&x) && (0.0..height).contains(&y) {
                    (self.on_click)();
                }

                self.is_down = false;
            }

            _ => (),
        }
        EventHandleResult::Pass
    }
}

pub struct ButtonBuilder<'a> {
    pub children: &'a [&'a dyn ComponentBuilder<'a>],
    pub on_click: Cell<Option<Box<dyn FnMut()>>>,
    pub style: taffy::Style,
}

impl_const_default!(
    ButtonBuilder<'_>,
    Self {
        children: &[],
        on_click: Cell::new(None),
        style: taffy_style! {
            size: taffy::Size {
                width: taffy::Dimension::length(100.0),
                height: taffy::Dimension::length(50.0)
            }
        }
    }
);

impl<'a> ComponentBuilder<'a> for ButtonBuilder<'a> {
    fn build(
        &self,
        _context: &mut BuilderContext,
    ) -> (Box<dyn ComponentTrait>, taffy::Style, Children<'a>) {
        (
            Box::new(Button {
                on_click: self.on_click.take().expect("On click function is gone! NOTE that ButtonBuilder is not idempotent"),
                is_down: false,
            }),
            self.style.clone(),
            Children::Borrowed(&self.children),
        )
    }
}
