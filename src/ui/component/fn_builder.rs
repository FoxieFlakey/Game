use std::marker::PhantomData;

use crate::ui::{
    BuilderContext,
    component::{Children, ComponentBuilder, ComponentTrait},
};

// Allows user to define arbitrary
// Fn to build a sub tree.
pub struct FnBuilder<'a, F>
    where
        F: Fn(&mut BuilderContext) -> (Box<dyn ComponentTrait>, taffy::Style, Children<'a>) + 'a,
{
    pub func: F,
    pub _phantom: PhantomData<&'a ()>
}

impl<'a, F> ComponentBuilder<'a> for FnBuilder<'a, F>
where
    F: Fn(&mut BuilderContext) -> (Box<dyn ComponentTrait>, taffy::Style, Children<'a>) + 'a
{
    fn build(
        &self,
        context: &mut BuilderContext,
    ) -> (Box<dyn ComponentTrait>, taffy::Style, Children<'a>) {
        (self.func)(context)
    }
}
