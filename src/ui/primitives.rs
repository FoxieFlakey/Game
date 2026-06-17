mod model;

pub use model::colored_rectangle::Instance as ColoredRectangle;
pub use model::colored_rectangle::render as render_colored_rectangle;

pub enum UIPrimitive {
    ColoredRectangle(ColoredRectangle),
}

pub fn init() {
    // Init models for primitives
    model::init();
}
