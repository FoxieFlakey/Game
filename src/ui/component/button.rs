use std::rc::Rc;

use taffy::style_helpers::TaffyAuto;

use crate::{
    events::{Event, EventHandleResult},
    ui::component::{ComponentBuilder, ComponentTrait},
    util::{impl_const_default, taffy_style},
};

pub struct Button {
    on_click: Option<Rc<dyn Fn()>>,
    is_down: bool
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
                    if let Some(func) = &self.on_click {
                        func();
                    }
                }
                
                self.is_down = false;
            },
            
            _ => ()
        }
        EventHandleResult::Pass
    }
}

pub struct ButtonBuilder<'a> {
    pub children: &'a [&'a dyn ComponentBuilder<'a>],
    pub on_click: Option<Rc<dyn Fn()>>,
    pub style: taffy::Style,
}

impl_const_default!(
    ButtonBuilder<'_>,
    Self {
        children: &[],
        on_click: None,
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
    ) -> (
        Box<dyn ComponentTrait>,
        taffy::Style,
        &'a [&'a dyn ComponentBuilder<'a>],
    ) {
        (
            Box::new(Button {
                on_click: self.on_click.clone(),
                is_down: false
            }),
            self.style.clone(),
            self.children,
        )
    }
}
