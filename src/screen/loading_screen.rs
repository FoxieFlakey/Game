use std::sync::LazyLock;

use glam::{Mat4, Quat, Vec3};
use smallvec::smallvec;

use crate::{events::EventHandleResult, rendering::{buffer::{BufferKind, VecBuf}, pipeline::VertexBufs}, screen::{Screen, loading_screen::resources::LoadingPawInstance}, states};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Camera {
    projection_matrix: Mat4
}

pub struct LoadingScreen {
    _camera_buffer: VecBuf<Camera>,
    camera_bind_group: wgpu::BindGroup,
    loading_paws: VecBuf<resources::LoadingPawInstance>
}

mod resources;

impl LoadingScreen {
    pub fn new() -> Self {
        // Explicitly make sure all of the resources loaded
        LazyLock::force(&resources::LOADING_PAW);
        LazyLock::force(&resources::LOADING_PAW_VIEW);
        LazyLock::force(&resources::LOADING_PAW_INDEX_BUFFER);
        LazyLock::force(&resources::LOADING_PAW_VERTEX);
        LazyLock::force(&resources::LOADING_PAW_BIND_GROUP_LAYOUT);
        LazyLock::force(&resources::LOADING_PAW_BIND_GROUP);
        LazyLock::force(&resources::LOADING_PAW_PIPELINE);
        LazyLock::force(&resources::LOADING_PAW_SAMPLER);
        LazyLock::force(&resources::CAMERA_BIND_GROUP_LAYOUT);
        
        let _camera_buffer = VecBuf::new_from_slice(
            states::main_dev::get().clone(),
            states::data_loader::get(),
            BufferKind::Uniform,
            &[
                Camera {
                    projection_matrix: Mat4::orthographic_lh(
                        0.0,
                        1280.0,
                        0.0,
                        720.0,
                        0.0,
                        1.0
                    )
                }
            ]
        );
        
        Self {
            loading_paws: VecBuf::new_from_slice(
                states::main_dev::get().clone(),
                states::data_loader::get(),
                BufferKind::Vertex,
                &[
                    // A single loading paw at bottom right
                    LoadingPawInstance {
                        transform: Mat4::from_scale_rotation_translation(
                            Vec3::splat(64.0),
                            Quat::IDENTITY,
                            Vec3 {
                                y: 32.0,
                                x: 1280.0 - 32.0,
                                z: 0.0
                            }
                        )
                    }
                ]
            ),
            camera_bind_group: states::main_dev::get()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &resources::CAMERA_BIND_GROUP_LAYOUT,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: _camera_buffer.as_binding()
                    }
                ]
            }),
            _camera_buffer,
        }
    }
}

impl Screen for LoadingScreen {
    fn handle_event(
        &mut self,
        _delta_time: std::time::Duration,
        _event: &sdl3::event::Event,
    ) -> anyhow::Result<crate::events::EventHandleResult>
    {
        Ok(EventHandleResult::Consumed)
    }
    
    fn render(
        &mut self,
        _delta_time: std::time::Duration,
        output_view: &wgpu::TextureView,
        cmd_encoder_creator: &dyn Fn(&wgpu::CommandEncoderDescriptor) -> wgpu::CommandEncoder,
    ) -> anyhow::Result<smallvec::SmallVec<[wgpu::CommandBuffer; super::STACK_ALLOCATED_COUNT]>>
    {
        let mut encoder = cmd_encoder_creator(&wgpu::CommandEncoderDescriptor::default());
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    store: wgpu::StoreOp::Store,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        a: 1.0,
                        b: 0.5,
                        g: 0.5,
                        r: 0.5
                    })
                }
            })],
            ..Default::default()
        });
        
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &*resources::LOADING_PAW_BIND_GROUP, &[]);
        resources::LOADING_PAW_PIPELINE
            .render(
                &mut render_pass,
                &VertexBufs {
                    buf0: Some(&resources::LOADING_PAW_VERTEX),
                    buf1: Some(&self.loading_paws),
                    ..Default::default()
                },
                &resources::LOADING_PAW_INDEX_BUFFER,
                0,
                0u32..resources::LOADING_PAW_INDEX_BUFFER.len() as u32,
                0u32..self.loading_paws.len() as u32,
                &[]
            );
        
        drop(render_pass);
        Ok(smallvec![ encoder.finish() ])
    }
}

