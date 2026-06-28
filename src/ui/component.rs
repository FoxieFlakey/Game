use std::time::Duration;

use glam::Mat4;

use crate::{events, ui::primitives::UIPrimitive};

mod rectangle;
pub use rectangle::Rectangle;

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
    // it would place at top left of component
    // on where its suppose be. For root node
    // it would be top left of screen. the Z axis
    // would be offseted properly
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
    // NOTE: coordinates on events ARE NOT translated
    // use invert transform matrix to turn it into local
    // coordinate
    fn handle_event(
        &mut self,
        invert_transform_matrix: Mat4,
        width: f32,
        height: f32,
        delta_time: Duration,
    ) -> events::EventHandleResult;
}
