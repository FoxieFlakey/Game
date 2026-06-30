use crate::{
    ui::component::{ComponentBuilder, ComponentTrait},
    util::{impl_const_default, taffy_style},
};

pub struct Column;

impl ComponentTrait for Column {}

pub struct ColumnBuilder<'a> {
    pub children: &'a [&'a dyn ComponentBuilder<'a>],
    pub style: taffy::Style,
}

impl_const_default!(
    ColumnBuilder<'_>,
    Self {
        children: &[],
        style: taffy_style! {
            display: taffy::Display::Flex,
            align_items: Some(taffy::AlignItems::Center),
            justify_content: Some(taffy::JustifyContent::Center),
            flex_direction: taffy::FlexDirection::Column,
            size: taffy::Size {
                width: taffy::Dimension::percent(1.0),
                height: taffy::Dimension::percent(1.0)
            }
        }
    }
);

impl<'a> ComponentBuilder<'a> for ColumnBuilder<'a> {
    fn build(
        &self,
    ) -> (
        Box<dyn ComponentTrait>,
        taffy::Style,
        &'a [&'a dyn ComponentBuilder<'a>],
    ) {
        (Box::new(Column), self.style.clone(), self.children)
    }
}
