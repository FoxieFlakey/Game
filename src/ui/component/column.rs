use crate::ui::component::Component;

pub struct Column;

impl Component for Column {
    fn get_base_style(&self) -> taffy::Style {
        taffy::Style {
            display: taffy::Display::Flex,
            align_items: Some(taffy::AlignItems::Center),
            justify_content: Some(taffy::JustifyContent::Center),
            flex_direction: taffy::FlexDirection::Column,
            size: taffy::Size {
                width: taffy::Dimension::percent(1.0),
                height: taffy::Dimension::percent(1.0)
            },
            ..Default::default()
        }
    }

    fn handle_event(
        &mut self,
        _transform_matrix: glam::Mat4,
        _width: f32,
        _height: f32,
        _delta_time: std::time::Duration,
        _event: crate::events::Event,
    ) -> crate::events::EventHandleResult {
        crate::events::EventHandleResult::Pass
    }

    fn render(
        &mut self,
        _transform_matrix: glam::Mat4,
        _width: f32,
        _height: f32,
        _delta_time: std::time::Duration,
        _primitive_collector: &mut dyn FnMut(crate::ui::primitives::UIPrimitive),
    ) {
    }
}
