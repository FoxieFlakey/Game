use std::sync::LazyLock;

use glam::{Mat4, Vec3, Vec4};

use crate::util::static_gpu_buffer;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    coord: Vec3
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub transform: Mat4,
    pub color: Vec4,
    pub width: u32,
    pub height: u32,
    pub _padding: [u8; 8]
}

static_gpu_buffer!(
    static Vertex VERTEX_BUFFER: LazyLock<VecBuf<[Vertex]>> => [
        // Bottom left
        Vertex { coord: Vec3 { x: -1.0, y: -1.0, z: 0.0 } },
        // Bottom right
        Vertex { coord: Vec3 { x:  1.0, y: -1.0, z: 0.0 } },
        // Top right
        Vertex { coord: Vec3 { x:  1.0, y:  1.0, z: 0.0 } },
        // Top left
        Vertex { coord: Vec3 { x: -1.0, y:  1.0, z: 0.0 } },
    ];
    
    static Index INDEX_BUFFER: LazyLock<VecBuf<[u16]>> => [
        0, 1, 3,
        3, 1, 2
    ];
);

pub fn init() {
    LazyLock::force(&VERTEX_BUFFER);
    LazyLock::force(&INDEX_BUFFER);
}



