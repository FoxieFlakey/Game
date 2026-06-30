use glam::{Mat4, Vec3, Vec4};

use crate::{
    events::{self, EventHandleResult},
    ui::{
        component::{ComponentBuilder, ComponentTrait},
        primitives::{self, UIPrimitive},
    }, util::{impl_const_default, taffy_style},
};

pub struct Rectangle {
    color: Vec4
}

impl ComponentTrait for Rectangle {
    fn handle_event(
        &mut self,
        _transform_matrix: glam::Mat4,
        width: f32,
        height: f32,
        _delta_time: std::time::Duration,
        event: events::Event,
    ) -> crate::events::EventHandleResult {
        match event {
            events::Event::MouseDown { x, y, .. } => {
                if !(0.0..width).contains(&x) {
                    return EventHandleResult::Pass;
                }
                
                if !(0.0..height).contains(&y) {
                    return EventHandleResult::Pass;
                }
                
                println!("Click at {x}, {y}");
            }

            events::Event::MouseUp { x, y, .. } => {
                if !(0.0..width).contains(&x) {
                    return EventHandleResult::Pass;
                }
                
                if !(0.0..height).contains(&y) {
                    return EventHandleResult::Pass;
                }
                
                println!("Click at {x}, {y}");
            }
        }

        EventHandleResult::Pass
    }

    fn render(
        &mut self,
        transform_matrix: glam::Mat4,
        width: f32,
        height: f32,
        _delta_time: std::time::Duration,
        primitive_collector: &mut dyn FnMut(UIPrimitive),
    ) {
        primitive_collector(UIPrimitive::ColoredRectangle(
            primitives::ColoredRectangle {
                color: self.color,
                transform: transform_matrix * Mat4::from_scale(Vec3::new(width, height, 1.0)),
            },
        ));
    }
}

pub struct RectangleBuilder {
    pub color: Vec4,
    pub style: taffy::Style
}

impl_const_default!(RectangleBuilder, RectangleBuilder {
    color: Vec4::ZERO,
    style: taffy_style! {
        size: taffy::Size {
            width: taffy::Dimension::percent(1.0),
            height: taffy::Dimension::percent(1.0),
        },
    }
});

impl<'a> ComponentBuilder<'a> for RectangleBuilder {
    fn build(&self) -> (Box<dyn ComponentTrait>, taffy::Style, &'a [&'a dyn ComponentBuilder<'a>]) {
        (Box::new(Rectangle { color: self.color }), self.style.clone(), &[])
    }
}
