use std::sync::LazyLock;

use glam::{Mat4, Vec2, Vec3};

use crate::{
    rendering::{buffer::VecBuf, pipeline::{Pipeline, VertexBufs, vertex_buffer_layout}},
    states,
    util::{identifier::Identifier, static_gpu_buffer},
};

pub struct LoadingPawModel {
    // These three need to be alive
    // its used indirectly from bind group
    _texture: wgpu::Texture,
    _view: wgpu::TextureView,
    _sampler: wgpu::Sampler,
    pipeline: Pipeline<u16, Vertex, LoadingPawInstance>,
    bind_group: wgpu::BindGroup,
}

impl LoadingPawModel {
    pub fn new() -> Self {
        let device = states::main_dev::get();
        let texture = states::early_registries::get()
            .textures
            .get(&Identifier::new("loading_icon"))
            .expect("Cannot find loading paw");
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../../resources/loading_screen.wgsl"));
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // The sampler which is required to
                // get texture's pixels .w.
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                },
                // The textures itself
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                },
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            min_filter: wgpu::FilterMode::Nearest,
            mag_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Make these initialized
        LazyLock::force(&LOADING_PAW_INDEX_BUFFER);
        LazyLock::force(&LOADING_PAW_VERTEX);

        Self {
            _texture: texture.clone(),
            bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                ],
            }),
            pipeline: Pipeline::new(
                device,
                &[],
                &shader,
                None,
                &shader,
                None,
                &[Some(wgpu::ColorTargetState {
                    format: *states::surface_format::get(),
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                Some(
                    &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Textures and data for 'loading paw' pipeline"),
                        immediate_size: 0,
                        bind_group_layouts: &[
                            Some(&super::CAMERA_BIND_GROUP_LAYOUT),
                            Some(&bind_group_layout),
                        ],
                    }),
                ),
            ),
            _sampler: sampler,
            _view: view,
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass, instances: &VecBuf<LoadingPawInstance>) {
        render_pass.set_bind_group(1, &self.bind_group, &[]);
        self.pipeline.render(
            render_pass,
            &VertexBufs {
                buf0: Some(&LOADING_PAW_VERTEX),
                buf1: Some(instances),
                ..Default::default()
            },
            &LOADING_PAW_INDEX_BUFFER,
            0,
            0u32..LOADING_PAW_INDEX_BUFFER.len() as u32,
            0u32..instances.len() as u32,
            &[],
        );
    }
}

pub static LOADING_PAW: LazyLock<LoadingPawModel> =
    LazyLock::new(|| LoadingPawModel::new());

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    coord: Vec3,
    tex_coord: Vec2,
}

vertex_buffer_layout!(Vertex as Vertex => [
    0 => Float32x3,
    1 => Float32x2
]);

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LoadingPawInstance {
    pub transform: Mat4,
}

vertex_buffer_layout!(LoadingPawInstance as Instance => [
    2 => Float32x4,
    3 => Float32x4,
    4 => Float32x4,
    5 => Float32x4
]);

static_gpu_buffer!(
    static Vertex LOADING_PAW_VERTEX: LazyLock<VecBuf<[Vertex]>> => [
        // Bottom left
        Vertex { coord: Vec3 { x: -0.5, y: -0.5, z: 0.0 }, tex_coord: Vec2 { x: 0.0, y: 1.0 } },
        // Bottom right
        Vertex { coord: Vec3 { x:  0.5, y: -0.5, z: 0.0 }, tex_coord: Vec2 { x: 1.0, y: 1.0 } },
        // Top right
        Vertex { coord: Vec3 { x:  0.5, y:  0.5, z: 0.0 }, tex_coord: Vec2 { x: 1.0, y: 0.0 } },
        // Top left
        Vertex { coord: Vec3 { x: -0.5, y:  0.5, z: 0.0 }, tex_coord: Vec2 { x: 0.0, y: 0.0 } },
    ];

    static Index LOADING_PAW_INDEX_BUFFER: LazyLock<VecBuf<[u16]>> => [
        0, 1, 3,
        3, 1, 2
    ];
);
