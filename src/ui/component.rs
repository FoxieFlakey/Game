use std::time::Duration;

use glam::Mat4;

use crate::{events, ui::primitives::UIPrimitive};

mod rectangle;
pub use rectangle::Rectangle;

mod row;
pub use row::Row;

mod column;
pub use column::Column;

pub trait Component {
    // To user, min and max size of a component
    // MUST not be modified to be less restrictive
    // like decreasing min and increasing max. Or
    // removing min/max limit
    fn get_base_style(&self) -> taffy::Style;

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
    );

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
    ) -> events::EventHandleResult;
}
