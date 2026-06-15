mod model;

pub use model::colored_rectangle::Instance as ColoredRectangle;

pub enum UIPrimitive {
    ColoredRectangle(ColoredRectangle)
}

pub fn init() {
    // Init models for primitives
    model::init();
}

