use glam::Mat4;

pub enum UIPrimitive {
    ColoredRectangle(ColoredRectangle)
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColoredRectangle {
    pub transform: Mat4,
    pub color: Color,
    pub width: u32,
    pub height: u32,
    pub _padding: [u8; 8]
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}


