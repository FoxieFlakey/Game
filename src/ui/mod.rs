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
    projection_matrix: Mat4,
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
            projection_matrix: Mat4::IDENTITY,
            camera,
        };
        this.on_resize(screen_width, screen_height);
        this
    }

    fn iter_tree<F>(&self, mut consumer: F)
    // Return true to quit iteration early
    where
        F: FnMut(
            /* Transform matrix */ Mat4,
            /* width */ f32,
            /* height */ f32,
            /* component */ &RefCell<Box<dyn Component>>,
        ) -> bool,
    {
        struct State<'a> {
            parent_transform_matrix: Mat4,
            parent_height: f32,
            childs: Box<dyn Iterator<Item = taffy::NodeId> + 'a>,
        }

        let root_layout = self.taffy.layout(self.root_id).unwrap();
        let root = self.taffy.get_node_context(self.root_id).unwrap();

        let do_ret = consumer(
            Mat4::from_translation(Vec3::new(
                root_layout.content_box_x(),
                // Gemini AI slop generated to uhh do whatever to translate
                // Taffy's 0,0 on top to WGPU's 0,0 at bottom left
                self.screen_height - root_layout.content_box_y() - root_layout.content_box_height(),
                0.0,
            )),
            root_layout.content_box_width(),
            root_layout.content_box_height(),
            root,
        );

        if do_ret {
            return;
        }

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

            let child_cell = self.taffy.get_node_context(child).unwrap();

            let do_ret = consumer(
                child_matrix,
                layout.content_box_width(),
                layout.content_box_height(),
                child_cell,
            );

            if do_ret {
                break;
            }

            // Push current child, so its child can be iterated
            stack.push(State {
                parent_transform_matrix: child_matrix,
                childs: Box::new(self.taffy.child_ids(child)),
                parent_height: layout.content_box_height(),
            });
        }
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
        let translated;
        match event {
            sdl3::event::Event::MouseButtonDown {
                mouse_btn: button,
                x,
                y,
                ..
            } => {
                translated = crate::events::Event::MouseDown {
                    x: *x,
                    // Flip the Y axis, so its starting from
                    // bottom left. Needed because screen coordinate
                    // is in term of WGPU's way so coords are inverted
                    // correctly
                    y: self.screen_height - *y,
                    button: *button,
                };
            }
            sdl3::event::Event::MouseButtonUp {
                mouse_btn: button,
                x,
                y,
                ..
            } => {
                translated = crate::events::Event::MouseUp {
                    x: *x,
                    // Flip the Y axis, so its starting from
                    // bottom left. Needed because screen coordinate
                    // is in term of WGPU's way so coords are inverted
                    // correctly
                    y: self.screen_height - *y,
                    button: *button,
                };
            }
            _ => return Ok(EventHandleResult::Pass),
        }

        self.iter_tree(|transform, width, height, node| {
            let transform_inverse = transform.inverse();
            let mut event = translated.clone();
            event.transform_coords(transform_inverse);
            let result = node
                .borrow_mut()
                .handle_event(transform, width, height, delta_time, event);

            // If event consumed, stop handling it
            matches!(result, EventHandleResult::Consumed)
        });

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

        let mut primitives = Vec::new();

        // PHASE 1: Traverse entire tree and collect all ui primitives
        let mut pusher = |x| primitives.push(x);
        self.iter_tree(|transform, width, height, node| {
            node.borrow_mut()
                .render(transform, width, height, delta_time, &mut pusher);
            false
        });

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

        self.projection_matrix =
            Mat4::orthographic_lh(0.0, new_screen_width, 0.0, new_screen_height, 0.0, 100000.0);

        self.camera.set(
            0,
            states::data_loader::get(),
            &Camera {
                projection_matrix: self.projection_matrix,
            },
        );
    }
}
