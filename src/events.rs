use glam::{Mat4, Vec4};

pub enum EventHandleResult {
    Consumed,
    Pass,
}

#[derive(Clone)]
pub enum Event {
    MouseDown {
        x: f32,
        y: f32,
        button: sdl3::mouse::MouseButton,
    },

    MouseUp {
        x: f32,
        y: f32,
        button: sdl3::mouse::MouseButton,
    },
}

impl Event {
    pub fn transform_coords(&mut self, transform: Mat4) {
        match self {
            Event::MouseDown { x, y, .. } | Event::MouseUp { x, y, .. } => {
                let mut coord = Vec4::new(*x, *y, 0.0, 1.0);
                coord = transform * coord;
                *x = coord.x;
                *y = coord.y;
            }
        }
    }
}
