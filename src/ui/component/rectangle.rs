use glam::{Mat4, Vec3, Vec4};

use crate::{
    events::{self, EventHandleResult},
    ui::{
        component::Component,
        primitives::{self, UIPrimitive},
    },
};

pub struct Rectangle;

impl Component for Rectangle {
    fn handle_event(
        &mut self,
        transform_matrix: glam::Mat4,
        width: f32,
        height: f32,
        delta_time: std::time::Duration,
        event: events::Event,
    ) -> crate::events::EventHandleResult {
        match event {
            events::Event::MouseDown { x, y, button } => {
                println!("Down at {x}, {y}");
            }

            events::Event::MouseUp { x, y, button } => {
                println!("Up at {x}, {y}");
            }
        }

        EventHandleResult::Pass
    }

    fn get_base_style(&self) -> taffy::Style {
        taffy::Style {
            padding: taffy::Rect {
                left: taffy::LengthPercentage::percent(0.01),
                right: taffy::LengthPercentage::percent(0.2),
                top: taffy::LengthPercentage::percent(0.01),
                bottom: taffy::LengthPercentage::percent(0.1),
            },
            size: taffy::Size {
                width: taffy::Dimension::percent(1.0),
                height: taffy::Dimension::percent(1.0),
            },
            ..Default::default()
        }
    }

    fn render(
        &mut self,
        transform_matrix: glam::Mat4,
        width: f32,
        height: f32,
        delta_time: std::time::Duration,
        primitive_collector: &mut dyn FnMut(UIPrimitive),
    ) {
        primitive_collector(UIPrimitive::ColoredRectangle(
            primitives::ColoredRectangle {
                color: Vec4::new(1.0, 0.0, 1.0, 1.0),
                transform: transform_matrix * Mat4::from_scale(Vec3::new(width, height, 1.0)),
            },
        ));
    }
}
