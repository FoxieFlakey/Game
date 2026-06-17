use std::sync::LazyLock;

use glam::{Mat4, Vec2, Vec3};

use crate::{rendering::pipeline::{Pipeline, vertex_buffer_layout}, states, util::static_gpu_buffer};

pub static LOADING_PAW: LazyLock<wgpu::Texture> = LazyLock::new(|| {
    let texture = image::load_from_memory(include_bytes!("../../resources/Loading paw.png"))
        .expect("Failed to load paw textures");
    
    states::data_loader::get().load_texture(texture)
});

pub static LOADING_PAW_VIEW: LazyLock<wgpu::TextureView> = LazyLock::new(|| {
    LOADING_PAW.create_view(&wgpu::TextureViewDescriptor::default())
});

pub static LOADING_PAW_SAMPLER: LazyLock<wgpu::Sampler> = LazyLock::new(|| {
    states::main_dev::get()
        .create_sampler(&wgpu::SamplerDescriptor {
            min_filter: wgpu::FilterMode::Nearest,
            mag_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        })
});

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    coord: Vec3,
    tex_coord: Vec2
}

vertex_buffer_layout!(Vertex as Vertex => [
    0 => Float32x3,
    1 => Float32x2
]);

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LoadingPawInstance {
    pub transform: Mat4
}

vertex_buffer_layout!(LoadingPawInstance as Instance => [
    2 => Float32x4,
    3 => Float32x4,
    4 => Float32x4,
    5 => Float32x4
]);

static_gpu_buffer!(
    pub static Vertex LOADING_PAW_VERTEX: LazyLock<VecBuf<[Vertex]>> => [
        // Bottom left
        Vertex { coord: Vec3 { x: -0.5, y: -0.5, z: 0.0 }, tex_coord: Vec2 { x: 0.0, y: 1.0 } },
        // Bottom right
        Vertex { coord: Vec3 { x:  0.5, y: -0.5, z: 0.0 }, tex_coord: Vec2 { x: 1.0, y: 1.0 } },
        // Top right
        Vertex { coord: Vec3 { x:  0.5, y:  0.5, z: 0.0 }, tex_coord: Vec2 { x: 1.0, y: 0.0 } },
        // Top left
        Vertex { coord: Vec3 { x: -0.5, y:  0.5, z: 0.0 }, tex_coord: Vec2 { x: 0.0, y: 0.0 } },
    ];

    pub static Index LOADING_PAW_INDEX_BUFFER: LazyLock<VecBuf<[u16]>> => [
        0, 1, 3,
        3, 1, 2
    ];
);

pub static LOADING_PAW_PIPELINE: LazyLock<Pipeline<u16, Vertex, LoadingPawInstance>> = LazyLock::new(|| {
    let device = states::main_dev::get();
    let shader = device.create_shader_module(wgpu::include_wgsl!("../../resources/loading_screen.wgsl"));

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
        Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Textures and data for 'loading paw' pipeline"),
            immediate_size: 0,
            bind_group_layouts: &[
                Some(&CAMERA_BIND_GROUP_LAYOUT),
                Some(&LOADING_PAW_BIND_GROUP_LAYOUT)
            ]
        }))
    ) 
});

pub static LOADING_PAW_BIND_GROUP_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
    states::main_dev::get()
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        multisampled: false
                    },
                }
            ]
        })
});

pub static LOADING_PAW_BIND_GROUP: LazyLock<wgpu::BindGroup> = LazyLock::new(|| {
    states::main_dev::get()
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &LOADING_PAW_BIND_GROUP_LAYOUT,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&LOADING_PAW_SAMPLER),
                },
                
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&LOADING_PAW_VIEW),
                }
            ]
        })
});

pub static CAMERA_BIND_GROUP_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
    states::main_dev::get()
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // The sampler which is required to
                // get texture's pixels .w.
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                },
            ]
        })
});

