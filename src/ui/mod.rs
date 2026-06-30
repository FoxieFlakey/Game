use std::{cell::RefCell, sync::LazyLock, time::Duration};

use glam::{Mat4, Vec3};
use smallvec::{SmallVec, smallvec};
use taffy::TraversePartialTree;

use crate::{
    events::EventHandleResult,
    rendering::buffer::{BufferKind, VecBuf},
    screen::Screen,
    states,
    ui::{component::{Component, ComponentBuilder, ComponentTrait}, primitives::UIPrimitive},
    util::vec_buf2,
};

pub mod component;
pub mod primitives;

pub struct UI {
    screen_width: f32,
    screen_height: f32,
    camera_bind_group: wgpu::BindGroup,
    camera: VecBuf<Camera>,
    taffy: taffy::TaffyTree<RefCell<Component>>,
    root_id: taffy::NodeId,
    projection_matrix: Mat4,
    colored_rects: Option<VecBuf<primitives::ColoredRectangle>>,
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
    pub fn new<'a, T: ComponentBuilder<'a> + 'a>(
        screen_width: f32,
        screen_height: f32,
        root_builder: &'a T,
    ) -> Self {
        let mut taffy = taffy::TaffyTree::new();

        let camera = vec_buf2!(
            Uniform,
            [Camera {
                projection_matrix: Mat4::IDENTITY
            }]
        );
        
        let mut this = Self {
            screen_width,
            screen_height,
            // Fake placeholder leaf, going to be replaced after Self is constructed
            root_id: taffy.new_leaf(taffy::Style::DEFAULT).unwrap(),
            taffy,
            colored_rects: Some(VecBuf::new(states::main_dev::get().clone(), BufferKind::Vertex)),
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
        let new_root = this.build_component(root_builder);
        this.taffy.remove(this.root_id).unwrap();
        this.root_id = new_root;
        
        this.on_resize(screen_width, screen_height);
        this
    }

    pub fn get_root_node(&self) -> taffy::NodeId {
        self.root_id
    }

    fn build_component<'a>(&mut self, builder: &'a dyn ComponentBuilder<'a>) -> taffy::NodeId {
        let (component, style, children) = builder.build();
        
        let component_id = self
            .taffy
            .new_leaf(style)
            .unwrap();
        
        self.taffy.set_node_context(
                component_id,
                Some(RefCell::new(Component {
                    node_id: component_id,
                    component
                })),
            )
            .unwrap();
        
        for child_builder in children {
            let child = self.build_component(*child_builder);
            self.taffy.add_child(component_id, child).unwrap();
        }
        
        component_id
    }
    
    pub fn add_child<'a, T: ComponentBuilder<'a> + 'a>(&mut self, parent: taffy::NodeId, builder: &'a T) -> taffy::NodeId {
        let child = self.build_component(builder);
        self.taffy.add_child(parent, child).unwrap();
        child
    }

    pub fn add_child_built<T: ComponentTrait + 'static>(&mut self, root: taffy::NodeId, style: taffy::Style, component: T) -> taffy::NodeId {
        let child = self
            .taffy
            .new_leaf(style)
            .unwrap();
        
        self.taffy.set_node_context(
                child,
                Some(RefCell::new(Component {
                    node_id: child,
                    component: Box::new(component)
                })),
            )
            .unwrap();
        self.taffy.add_child(root, child).unwrap();
        child
    }

    fn iter_tree<F>(&self, mut consumer: F)
    // Return true to quit iteration early
    where
        F: FnMut(
            /* Transform matrix */ Mat4,
            /* width */ f32,
            /* height */ f32,
            /* component */ &RefCell<Component>,
        ) -> bool,
    {
        struct State<'a> {
            parent_transform_matrix: Mat4,
            parent_height: f32,
            childs: Box<dyn Iterator<Item = taffy::NodeId> + 'a>,
        }

        let root_layout = self.taffy.layout(self.root_id).unwrap();
        let root = self.taffy.get_node_context(self.root_id).unwrap();

        let parent_transform_matrix = Mat4::from_translation(Vec3::new(
                root_layout.content_box_x(),
                // Gemini AI slop generated to uhh do whatever to translate
                // Taffy's 0,0 on top to WGPU's 0,0 at bottom left
                self.screen_height - root_layout.content_box_y() - root_layout.content_box_height(),
                0.0,
            ));
        let do_ret = consumer(
            parent_transform_matrix,
            root_layout.content_box_width(),
            root_layout.content_box_height(),
            root,
        );

        if do_ret {
            return;
        }

        let mut stack = Vec::new();
        stack.push(State {
            parent_transform_matrix,
            childs: Box::new(self.taffy.child_ids(self.root_id)),
            parent_height: root_layout.content_box_height(),
        });

        loop {
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
                        - layout.content_box_height()
                        - layout.content_box_y(),
                    1.0,
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

        // PHASE 1: Traverse entire tree and collect all ui primitives
        let mut colored_rects = self.colored_rects.take().unwrap();
        colored_rects.clear();
        
        let data_loader = states::data_loader::get();
        let mut pusher = |x| {
            match x {
                UIPrimitive::ColoredRectangle(x) => colored_rects.push(data_loader, x)
            }
        };
        
        self.iter_tree(|transform, width, height, node| {
            node.borrow_mut()
                .render(transform, width, height, delta_time, &mut pusher);
            false
        });
        
        self.colored_rects = Some(colored_rects);

        // Phase 3: Render all primitives
        let mut encoder = cmd_encoder_creator(&wgpu::CommandEncoderDescriptor::default());
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                depth_slice: None,
                view: output_view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
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
            self.colored_rects.as_ref().unwrap(),
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
