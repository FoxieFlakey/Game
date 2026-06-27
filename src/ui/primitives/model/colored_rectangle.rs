use std::sync::LazyLock;

use glam::{Mat4, Vec3, Vec4};

use crate::{
    rendering::{
        buffer::VecBuf,
        pipeline::{Pipeline, VertexBufs, vertex_buffer_layout},
    },
    states,
    util::{identifier::Identifier, static_gpu_buffer},
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    coord: Vec3,
}

vertex_buffer_layout!(Vertex as Vertex => [
    0 => Float32x3
]);

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub transform: Mat4,
    pub color: Vec4,
}

vertex_buffer_layout!(Instance as Instance => [
    // transform matrix split into 4 locations
    1 => Float32x4,
    2 => Float32x4,
    3 => Float32x4,
    4 => Float32x4,

    // And color
    5 => Float32x4,
]);

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

static PIPELINE: LazyLock<Pipeline<u16, Vertex, Instance>> = LazyLock::new(|| {
    let device = states::main_dev::get();
    let shader = states::registries::get()
        .shaders
        .get(&Instance::SHADER_ID)
        .expect("Cannot find shader for colored rectangle");
    Pipeline::new(
        device,
        &[],
        &shader,
        None,
        &shader,
        None,
        &[Some(wgpu::ColorTargetState {
            format: *states::surface_format::get(),
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })],
        None,
    )
});

impl Instance {
    pub const SHADER_ID: Identifier = Identifier::new_const("ui/colored_rectangle");
}

pub fn render(render_pass: &mut wgpu::RenderPass, instances: &VecBuf<Instance>) {
    PIPELINE.render(
        render_pass,
        &VertexBufs {
            buf0: Some(&VERTEX_BUFFER),
            buf1: Some(instances),
            ..Default::default()
        },
        &INDEX_BUFFER,
        0,
        0u32..INDEX_BUFFER.len() as u32,
        0u32..instances.len() as u32,
        &[],
    );
}

pub fn init() {
    LazyLock::force(&VERTEX_BUFFER);
    LazyLock::force(&INDEX_BUFFER);
    LazyLock::force(&PIPELINE);
}
