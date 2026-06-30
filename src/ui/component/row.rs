use crate::ui::component::{ComponentTrait, ComponentBuilder};

pub struct Row;

impl ComponentTrait for Row {
    fn get_base_style(&self) -> taffy::Style {
        taffy::Style {
            display: taffy::Display::Flex,
            align_items: Some(taffy::AlignItems::Center),
            justify_content: Some(taffy::JustifyContent::Center),
            flex_direction: taffy::FlexDirection::Row,
            size: taffy::Size {
                width: taffy::Dimension::percent(1.0),
                height: taffy::Dimension::percent(1.0)
            },
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct RowBuilder<'a> {
    pub children: &'a [&'a dyn ComponentBuilder<'a>]
}

impl<'a> ComponentBuilder<'a> for RowBuilder<'a> {
    fn build(&self) -> (Box<dyn ComponentTrait>, &'a [&'a dyn ComponentBuilder<'a>]) {
        (Box::new(Row), self.children)
    }
}
