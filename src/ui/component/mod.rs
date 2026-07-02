use std::any::Any;
use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::time::Duration;

use glam::Mat4;

use crate::ui::BuilderContext;
use crate::{events, ui::primitives::UIPrimitive};

mod rectangle;
pub use rectangle::Rectangle;
pub use rectangle::RectangleBuilder;

mod row;
pub use row::Row;
pub use row::RowBuilder;

mod column;
pub use column::Column;
pub use column::ColumnBuilder;

mod button;
pub use button::Button;
pub use button::ButtonBuilder;

mod fn_builder;
pub use fn_builder::FnBuilder;

pub enum Children<'a> {
    // Borrowed builders
    Borrowed(&'a [&'a dyn ComponentBuilder<'a>]),

    // Represent built childrens that was eagerly built
    // there no borrow variants, as these has to be
    // newly built childs
    Built(Vec<taffy::NodeId>),

    // There no children
    None,
}

pub trait ComponentBuilder<'a> {
    // returns the built component
    // and the component's styling
    // and the children builders
    //
    // Callin again would give same values
    fn build(
        &self,
        _context: &mut BuilderContext,
    ) -> (Box<dyn ComponentTrait>, taffy::Style, Children<'a>);
}

pub trait ComponentTrait: Any {
    // Transform matrix is matrix saying how
    // current component is transformed
    //
    // Its already calculated so given 0, 0
    // it would place at bottom left of component
    // on where its suppose be. For root node
    // it would be bottom left of screen. the Z axis
    // would be offseted properly. Width and height is the
    // size of component at time of render
    //
    // containers DO NOT need to render childs
    // the UI handles that
    fn render(
        &mut self,
        transform_matrix: Mat4,
        width: f32,
        height: f32,
        delta_time: Duration,
        primitive_collector: &mut dyn FnMut(UIPrimitive),
    ) {
        let _ = transform_matrix;
        let _ = width;
        let _ = height;
        let _ = delta_time;
        let _ = primitive_collector;
    }

    // These receives borrow to the same UI
    // that the components attached to
    //
    // Returning Consumed, will prevent the event
    // traverse to deeper. Width and height is the
    // size of component at time of handle_event
    //
    // NOTE: coordinates on events is already transformed
    // properly, so 0, 0 is bottom left of current component
    //
    // the transform matrix given, so callee can turn it
    // to screen coordinate as necessary. NOTE the screen
    // coordinate is starting at bottom left! different
    // than one provided from raw SDL coords
    fn handle_event(
        &mut self,
        transform_matrix: Mat4,
        width: f32,
        height: f32,
        delta_time: Duration,
        event: events::Event,
    ) -> events::EventHandleResult {
        let _ = transform_matrix;
        let _ = width;
        let _ = height;
        let _ = delta_time;
        let _ = event;

        events::EventHandleResult::Pass
    }
}

// A concrete container for component
// (it still mostly passthru methods to underlying one)
pub struct Component {
    pub(super) node_id: taffy::NodeId,
    pub(super) component: RefCell<Box<dyn ComponentTrait>>,
}

impl Component {
    // Return Taffy's NodeID that refer to this
    // component
    pub fn get_node_id(&self) -> taffy::NodeId {
        self.node_id
    }

    pub fn borrow<'a>(&'a self) -> Ref<'a, Box<dyn ComponentTrait>> {
        self.component.borrow()
    }

    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a, Box<dyn ComponentTrait>> {
        self.component.borrow_mut()
    }
}
