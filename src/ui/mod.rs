use std::{cell::RefCell, sync::LazyLock, time::Duration};

use glam::{Mat4, Vec3};
use smallvec::{SmallVec, smallvec};
use taffy::TraversePartialTree;

use crate::{
    events::EventHandleResult,
    rendering::buffer::{BufferKind, VecBuf},
    screen::Screen,
    states,
    ui::{component::Component, primitives::UIPrimitive},
    util::vec_buf2,
};

pub mod component;
pub mod primitives;

pub struct UI {
    screen_width: f32,
    screen_height: f32,
    camera_bind_group: wgpu::BindGroup,
    camera: VecBuf<Camera>,
    taffy: taffy::TaffyTree<RefCell<Box<dyn Component>>>,
    root_id: taffy::NodeId,

    colored_rects: VecBuf<primitives::ColoredRectangle>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Camera {
    projection_matrix: Mat4,
}

static CAMERA_BIND_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
    states::main_dev::get().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            count: None,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                has_dynamic_offset: false,
                min_binding_size: None,
                ty: wgpu::BufferBindingType::Uniform,
            },
        }],
    })
});

impl UI {
    pub fn new<T: Component + 'static>(
        screen_width: f32,
        screen_height: f32,
        root_component: T,
    ) -> Self {
        let mut taffy: taffy::TaffyTree<RefCell<Box<dyn Component>>> = taffy::TaffyTree::new();

        let root_id = taffy.new_leaf(root_component.get_base_style()).unwrap();
        taffy
            .set_node_context(root_id, Some(RefCell::new(Box::new(root_component))))
            .unwrap();
        let camera = vec_buf2!(
            Uniform,
            [Camera {
                projection_matrix: Mat4::IDENTITY
            }]
        );

        let mut this = Self {
            screen_width,
            screen_height,
            taffy,
            root_id,
            colored_rects: VecBuf::new(states::main_dev::get().clone(), BufferKind::Vertex),
            camera_bind_group: states::main_dev::get().create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &CAMERA_BIND_LAYOUT,
                    label: None,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera.as_binding(),
                    }],
                },
            ),
            camera,
        };
        this.on_resize(screen_width, screen_height);
        this
    }
}

pub fn init() -> anyhow::Result<()> {
    primitives::init();

    Ok(())
}

impl Screen for UI {
    fn handle_event(
        &mut self,
        delta_time: Duration,
        event: &sdl3::event::Event,
    ) -> anyhow::Result<EventHandleResult> {
        Ok(EventHandleResult::Pass)
    }

    fn render(
        &mut self,
        delta_time: Duration,
        output_view: &wgpu::TextureView,
        cmd_encoder_creator: &dyn Fn(&wgpu::CommandEncoderDescriptor) -> wgpu::CommandEncoder,
    ) -> anyhow::Result<SmallVec<[wgpu::CommandBuffer; crate::screen::STACK_ALLOCATED_COUNT]>> {
        self.taffy
            .compute_layout(
                self.root_id,
                taffy::Size {
                    width: taffy::AvailableSpace::Definite(self.screen_width),
                    height: taffy::AvailableSpace::Definite(self.screen_height),
                },
            )
            .unwrap();

        struct State<'a> {
            parent_transform_matrix: Mat4,
            parent_height: f32,
            childs: Box<dyn Iterator<Item = taffy::NodeId> + 'a>,
        }

        let mut primitives = Vec::new();

        // Render the root first
        // PHASE 1: Traverse entire tree
        // and collect all ui primitives

        let root_layout = self.taffy.layout(self.root_id).unwrap();
        self.taffy
            .get_node_context(self.root_id)
            .unwrap()
            .borrow_mut()
            .render(
                Mat4::from_translation(Vec3::new(
                    root_layout.content_box_x(),
                    // Gemini AI slop generated to uhh do whatever to translate
                    // Taffy's 0,0 on top to WGPU's 0,0 at bottom left
                    self.screen_height
                        - root_layout.content_box_y()
                        - root_layout.content_box_height(),
                    0.0,
                )),
                root_layout.content_box_width(),
                root_layout.content_box_height(),
                delta_time,
                &mut |x| primitives.push(x),
            );

        let mut stack = Vec::new();
        stack.push(State {
            parent_transform_matrix: Mat4::IDENTITY,
            childs: Box::new(self.taffy.child_ids(self.root_id)),
            parent_height: root_layout.content_box_height(),
        });

        loop {
            // Depth 0.0 is used by root before
            let depth = stack.len() + 1;
            let Some(top) = stack.last_mut() else {
                // Traversed all nodes starting from root
                break;
            };

            let Some(child) = top.childs.next() else {
                stack.pop();
                continue;
            };

            // Render current child
            let layout = self.taffy.layout(child).unwrap();
            let child_matrix = top.parent_transform_matrix
                * Mat4::from_translation(Vec3::new(
                    layout.content_box_x(),
                    // Gemini AI slop generated to uhh do whatever to translate
                    // Taffy's 0,0 on top to WGPU's 0,0 at bottom left
                    top.parent_height
                        - root_layout.content_box_height()
                        - root_layout.content_box_y(),
                    depth as f32,
                ));

            self.taffy
                .get_node_context(child)
                .unwrap()
                .borrow_mut()
                .render(
                    child_matrix,
                    layout.content_box_width(),
                    layout.content_box_height(),
                    delta_time,
                    &mut |x| primitives.push(x),
                );

            // Push current child, so its child can be iterated
            stack.push(State {
                parent_transform_matrix: child_matrix,
                childs: Box::new(self.taffy.child_ids(child)),
                parent_height: layout.content_box_height(),
            });
        }

        // Phase 2: Now fill the primitives into corresponding instance buffer :3
        self.colored_rects.clear();

        for primitive in primitives {
            match primitive {
                UIPrimitive::ColoredRectangle(x) => self
                    .colored_rects
                    .extend_from_slice(states::data_loader::get(), &[x]),
            }
        }

        // Phase 3: Render all primitives
        let mut encoder = cmd_encoder_creator(&wgpu::CommandEncoderDescriptor::default());
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                depth_slice: None,
                view: output_view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                    store: wgpu::StoreOp::Store,
                },
                resolve_target: None,
            })],
            label: Some("Render primitives"),
            ..Default::default()
        });

        primitives::render_colored_rectangle(
            &mut render_pass,
            &self.camera_bind_group,
            &self.colored_rects,
        );

        drop(render_pass);
        Ok(smallvec![encoder.finish()])
    }

    fn on_resize(&mut self, new_screen_width: f32, new_screen_height: f32) {
        self.screen_width = new_screen_width;
        self.screen_height = new_screen_height;

        self.camera.set(
            0,
            states::data_loader::get(),
            &Camera {
                // NOTE: in UI, 0.0 is top left
                // so the "height" coord is at bottom
                projection_matrix: Mat4::orthographic_lh(
                    0.0,
                    new_screen_width,
                    0.0,
                    new_screen_height,
                    0.0,
                    100000.0,
                ),
            },
        );
    }
}
