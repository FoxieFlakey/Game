use std::sync::LazyLock;

use glam::{Mat4, Quat, Vec3};
use smallvec::smallvec;

use crate::{
    events::EventHandleResult,
    rendering::buffer::{BufferKind, VecBuf},
    screen::{Screen, loading_screen::loading_paw_model::LoadingPawInstance},
    states,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Camera {
    projection_matrix: Mat4,
}

pub struct LoadingScreen {
    _camera_buffer: VecBuf<Camera>,
    camera_bind_group: wgpu::BindGroup,
    loading_paws: VecBuf<loading_paw_model::LoadingPawInstance>,
    rotation_deg: f32,
    scale: Vec3,
    translation: Vec3
}

mod loading_paw_model;

static CAMERA_BIND_GROUP_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
    states::main_dev::get().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    min_binding_size: None,
                },
            },
        ],
    })
});

impl LoadingScreen {
    pub fn new() -> Self {
        // Explicitly make sure all of the resources loaded
        LazyLock::force(&loading_paw_model::LOADING_PAW);
        LazyLock::force(&CAMERA_BIND_GROUP_LAYOUT);

        let _camera_buffer = VecBuf::new_from_slice(
            states::main_dev::get().clone(),
            states::data_loader::get(),
            BufferKind::Uniform,
            &[Camera {
                projection_matrix: Mat4::orthographic_lh(0.0, 1280.0, 0.0, 720.0, 0.0, 1.0),
            }],
        );

        let mut loading_paws = VecBuf::new_with_initial_capacity(
            states::main_dev::get().clone(),
            BufferKind::Vertex,
            1
        );
        loading_paws.resize(states::data_loader::get(), 1);
        
        Self {
            loading_paws,
            scale: Vec3::splat(120.0),
            translation: Vec3 {
                y: 70.0,
                x: 1280.0 - 70.0,
                z: 0.0,
            },
            camera_bind_group: states::main_dev::get().create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &CAMERA_BIND_GROUP_LAYOUT,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: _camera_buffer.as_binding(),
                    }],
                },
            ),
            _camera_buffer,
            rotation_deg: 0.0,
        }
    }
}

impl Screen for LoadingScreen {
    fn handle_event(
        &mut self,
        _delta_time: std::time::Duration,
        _event: &sdl3::event::Event,
    ) -> anyhow::Result<crate::events::EventHandleResult> {
        Ok(EventHandleResult::Consumed)
    }

    fn render(
        &mut self,
        delta_time: std::time::Duration,
        output_view: &wgpu::TextureView,
        cmd_encoder_creator: &dyn Fn(&wgpu::CommandEncoderDescriptor) -> wgpu::CommandEncoder,
    ) -> anyhow::Result<smallvec::SmallVec<[wgpu::CommandBuffer; super::STACK_ALLOCATED_COUNT]>>
    {
        self.rotation_deg -= delta_time.as_secs_f32() * 160.0;
        
        self.loading_paws.set(0, states::data_loader::get(), &LoadingPawInstance {
            transform: Mat4::from_scale_rotation_translation(
                self.scale,
                Quat::from_rotation_z(self.rotation_deg.to_radians()),
                self.translation
            )
        });
        
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
                        r: 0.5,
                    }),
                },
            })],
            ..Default::default()
        });

        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        loading_paw_model::LOADING_PAW.render(
            &mut render_pass,
            &self.loading_paws
        );

        drop(render_pass);
        Ok(smallvec![encoder.finish()])
    }
}
