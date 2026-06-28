use glam::{Vec2, Vec3};

use crate::{
    rendering::{
        buffer::{BufferKind, VecBuf},
        data_loader::DataLoader,
        pipeline::{Pipeline, VertexBufs, vertex_buffer_layout},
        util,
    },
    util::vec_buf,
};

pub struct Frame {
    pub output: wgpu::Texture,
    // This is necessary because bind_group needs this
    _texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

pub struct FrameBlitter {
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    pipeline: Pipeline<u16, Vertex>,
    device: wgpu::Device,
    uniforms: VecBuf<Uniforms>,
    data_loader: DataLoader,

    render_output_model: VecBuf<Vertex>,
    render_output_model_indexes: VecBuf<u16>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    // The width and height of the output
    // in window
    output_width: f32,
    output_height: f32,

    // The width and height of game
    // viewport
    render_width: f32,
    render_height: f32,
}

#[rustfmt::skip]
impl FrameBlitter {
    pub fn new(
        device: wgpu::Device,
        blit_shader: &wgpu::ShaderModule,
        output_format: wgpu::TextureFormat,
        data_loader: DataLoader,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // The sampler so the texture can be sampled and translated
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                },
                // The input framebuffer, which needs to be translated to
                // presentation surface's texture format
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                // Uniform buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    count: None,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
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

        let mut uniforms = VecBuf::new(device.clone(), BufferKind::Uniform);
        uniforms.extend_from_slice(
            &data_loader,
            &[Uniforms {
                output_height: 1.0,
                output_width: 1.0,
                render_height: 1.0,
                render_width: 1.0,
            }],
        );

        FrameBlitter {
            sampler,
            pipeline: Pipeline::new(
                &device,
                &[],
                &blit_shader,
                None,
                &blit_shader,
                None,
                &[Some(wgpu::ColorTargetState {
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    format: output_format,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                Some(
                    &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        immediate_size: 0,
                        bind_group_layouts: &[Some(&bind_group_layout)],
                    }),
                ),
            ),
            render_output_model: vec_buf!(
                device.clone(),
                &data_loader,
                Vertex,
                [
                   // Bottom left
                    Vertex { coord: Vec3 { x: -1.0, y: -1.0, z: 0.0 }, tex_coord: Vec2 { x: 0.0, y: 1.0 } },
                    // Bottom right
                    Vertex { coord: Vec3 { x:  1.0, y: -1.0, z: 0.0 }, tex_coord: Vec2 { x: 1.0, y: 1.0 } },
                    // Top right
                    Vertex { coord: Vec3 { x:  1.0, y:  1.0, z: 0.0 }, tex_coord: Vec2 { x: 1.0, y: 0.0 } },
                    // Top left
                    Vertex { coord: Vec3 { x: -1.0, y:  1.0, z: 0.0 }, tex_coord: Vec2 { x: 0.0, y: 0.0 } },
                ]
            ),
            render_output_model_indexes: vec_buf!(
                device.clone(),
                &data_loader,
                Index,
                [
                    0, 1, 3,
                    3, 1, 2
                ]
            ),
            uniforms,
            bind_group_layout,
            device,
            data_loader,
        }
    }

    pub fn present(
        &mut self,
        frame: &Frame,
        output_surface: &wgpu::SurfaceTexture,
        queue: &wgpu::Queue,
        output_width: u32,
        output_height: u32,
        render_width: u32,
        render_height: u32,
    ) {
        let mut cmd = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Blit final frame to display"),
            });

        let mut render_pass = cmd.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_surface
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default()),
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        self.uniforms.set(
            0,
            &self.data_loader,
            &Uniforms {
                output_width: output_width as f32,
                output_height: output_height as f32,
                render_width: render_width as f32,
                render_height: render_height as f32,
            },
        );

        render_pass.set_bind_group(0, &frame.bind_group, &[]);
        self.pipeline.render(
            &mut render_pass,
            &VertexBufs {
                buf0: Some(&self.render_output_model),
                ..Default::default()
            },
            &self.render_output_model_indexes,
            0,
            0..self.render_output_model_indexes.len() as u32,
            0..1,
            &[],
        );

        drop(render_pass);
        let id = queue.submit([cmd.finish()]);

        // Have to wait, or else if CPU fast enough it would reach the this function again before the
        // Texture for surface is done being used by GPU (to be blitted/mapped into)
        util::wait_device(&self.device, id);
    }

    pub fn new_frame(&self, texture: wgpu::Texture) -> Frame {
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Frame {
            output: texture,
            bind_group: self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                label: None,
                entries: &[
                    // Sampler for the texture
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    // The texture itself
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    // Uniforms
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.uniforms.as_binding(),
                    },
                ],
            }),
            _texture_view: texture_view,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    coord: Vec3,
    tex_coord: Vec2,
}

vertex_buffer_layout!(Vertex as Vertex => [
    0 => Float32x3,
    1 => Float32x2
]);
