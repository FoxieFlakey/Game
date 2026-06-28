use crate::ui::component::{Component, ComponentBuilder};

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
}

#[derive(Default)]
pub struct ColumnBuilder<'a> {
    pub children: &'a [&'a dyn ComponentBuilder<'a>]
}

impl<'a> ComponentBuilder<'a> for ColumnBuilder<'a> {
    fn build(&self) -> (Box<dyn Component>, &'a [&'a dyn ComponentBuilder<'a>]) {
        (Box::new(Column), self.children)
    }
}
