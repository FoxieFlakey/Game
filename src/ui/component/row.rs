use crate::{
    ui::component::{Children, ComponentBuilder, ComponentTrait},
    util::{impl_const_default, taffy_style},
};

pub struct Row;

impl ComponentTrait for Row {}

pub struct RowBuilder<'a> {
    pub children: &'a [&'a dyn ComponentBuilder<'a>],
    pub style: taffy::Style,
}

impl_const_default!(
    RowBuilder<'_>,
    Self {
        children: &[],
        style: taffy_style! {
            display: taffy::Display::Flex,
            align_items: Some(taffy::AlignItems::Center),
            justify_content: Some(taffy::JustifyContent::Center),
            flex_direction: taffy::FlexDirection::Row,
            size: taffy::Size {
                width: taffy::Dimension::percent(1.0),
                height: taffy::Dimension::percent(1.0)
            },
        }
    }
);

impl<'a> ComponentBuilder<'a> for RowBuilder<'a> {
    fn build(&self) -> (Box<dyn ComponentTrait>, taffy::Style, Children<'a>) {
        (
            Box::new(Row),
            self.style.clone(),
            Children::Borrowed(self.children),
        )
    }
}
