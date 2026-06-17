// This assumes input is both vertices and instances
// for statically checking to ensure correct buffer
// are given

use std::{marker::PhantomData, ops::Range};

mod macros;
pub(crate) use macros::vertex_buffer_layout;
use smallvec::SmallVec;

use crate::rendering::buffer::VecBuf;

pub struct Pipeline<
    Index: HasIndexFormat = (),
    // Extra buffers that can be attached, up to total
    // of 8 slots. If its (), then its not used. Additional
    // can be added but it won't be type checked at compile
    // time.
    Vertex0: HasVertexBufferLayout = (),
    Vertex1: HasVertexBufferLayout = (),
    Vertex2: HasVertexBufferLayout = (),
    Vertex3: HasVertexBufferLayout = (),
    Vertex4: HasVertexBufferLayout = (),
    Vertex5: HasVertexBufferLayout = (),
    Vertex6: HasVertexBufferLayout = (),
    Vertex7: HasVertexBufferLayout = (),
> {
    pipeline: wgpu::RenderPipeline,
    extra_bufs_count: usize,
    _phantom: PhantomData<(
        Index,
        Vertex0,
        Vertex1,
        Vertex2,
        Vertex3,
        Vertex4,
        Vertex5,
        Vertex6,
        Vertex7,
    )>,
}

pub struct VertexBufs<
    'a,
    Vertex0: HasVertexBufferLayout,
    Vertex1: HasVertexBufferLayout,
    Vertex2: HasVertexBufferLayout,
    Vertex3: HasVertexBufferLayout,
    Vertex4: HasVertexBufferLayout,
    Vertex5: HasVertexBufferLayout,
    Vertex6: HasVertexBufferLayout,
    Vertex7: HasVertexBufferLayout,
> {
    pub buf0: Option<&'a VecBuf<Vertex0>>,
    pub buf1: Option<&'a VecBuf<Vertex1>>,
    pub buf2: Option<&'a VecBuf<Vertex2>>,
    pub buf3: Option<&'a VecBuf<Vertex3>>,
    pub buf4: Option<&'a VecBuf<Vertex4>>,
    pub buf5: Option<&'a VecBuf<Vertex5>>,
    pub buf6: Option<&'a VecBuf<Vertex6>>,
    pub buf7: Option<&'a VecBuf<Vertex7>>,
}

impl<
    Vertex0: HasVertexBufferLayout,
    Vertex1: HasVertexBufferLayout,
    Vertex2: HasVertexBufferLayout,
    Vertex3: HasVertexBufferLayout,
    Vertex4: HasVertexBufferLayout,
    Vertex5: HasVertexBufferLayout,
    Vertex6: HasVertexBufferLayout,
    Vertex7: HasVertexBufferLayout,
> Default
    for VertexBufs<'_, Vertex0, Vertex1, Vertex2, Vertex3, Vertex4, Vertex5, Vertex6, Vertex7>
{
    fn default() -> Self {
        Self {
            buf0: None,
            buf1: None,
            buf2: None,
            buf3: None,
            buf4: None,
            buf5: None,
            buf6: None,
            buf7: None,
        }
    }
}

impl<
    Index: HasIndexFormat,
    Vertex0: HasVertexBufferLayout,
    Vertex1: HasVertexBufferLayout,
    Vertex2: HasVertexBufferLayout,
    Vertex3: HasVertexBufferLayout,
    Vertex4: HasVertexBufferLayout,
    Vertex5: HasVertexBufferLayout,
    Vertex6: HasVertexBufferLayout,
    Vertex7: HasVertexBufferLayout,
> Pipeline<Index, Vertex0, Vertex1, Vertex2, Vertex3, Vertex4, Vertex5, Vertex6, Vertex7>
{
    pub fn new(
        device: &wgpu::Device,
        // Extra vertex buffers that want to be attached if
        // a GPU supports more than 8 (minimum by wgpu spec)
        extra_vertex_bufs: &[wgpu::VertexBufferLayout<'_>],
        vertex_shader: &wgpu::ShaderModule,
        vertex_shader_entry: Option<&str>,
        fragment_shader: &wgpu::ShaderModule,
        fragment_shader_entry: Option<&str>,
        fragment_targets: &[Option<wgpu::ColorTargetState>],
    ) -> Self {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let mut bufs = SmallVec::<[_; 8]>::new();

        macro_rules! append_layout {
            ($name:ident) => {
                if $name::is_something(private::Internal) {
                    bufs.push($name::LAYOUT);
                }
            };
        }

        append_layout!(Vertex0);
        append_layout!(Vertex1);
        append_layout!(Vertex2);
        append_layout!(Vertex3);
        append_layout!(Vertex4);
        append_layout!(Vertex5);
        append_layout!(Vertex6);
        append_layout!(Vertex7);

        bufs.reserve(extra_vertex_bufs.len());
        bufs.extend(extra_vertex_bufs.iter().cloned());

        Self {
            _phantom: PhantomData,
            pipeline: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: vertex_shader,
                    entry_point: vertex_shader_entry,
                    buffers: &bufs,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: fragment_shader,
                    entry_point: fragment_shader_entry,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: fragment_targets,
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                cache: None,
                multiview_mask: None,
                multisample: wgpu::MultisampleState {
                    alpha_to_coverage_enabled: false,
                    count: 1,
                    mask: !0,
                },
            }),
            extra_bufs_count: extra_vertex_bufs.len(),
        }
    }

    fn render_impl(
        &self,
        render_pass: &mut wgpu::RenderPass,
        vertex_bufs: &VertexBufs<
            Vertex0,
            Vertex1,
            Vertex2,
            Vertex3,
            Vertex4,
            Vertex5,
            Vertex6,
            Vertex7,
        >,
        index_buf_slice: Option<wgpu::BufferSlice<'_>>,

        // Vertices range is range in vertices that will be drawn
        // NOT the vertices in vertex buffer. In indexed rendering
        // it would be mapped to range in index buffer. In non indexed
        // it maps to range in vertex buffer
        vertices_range: Range<u32>,
        instance_range: Range<u32>,

        // only used in indexed rendering
        // specifies it adds to every vertex
        // index in index buffer before used
        // to find vertex in vertex buffer
        base_vertex: Option<i32>,

        // Additional buffer that won't be typechecked
        extra_vertex_bufs: &[&wgpu::Buffer],
    ) {
        macro_rules! check_vertex_buf {
            ($name:ident, $field_name:ident, $idx_var:ident) => {
                if $name::is_something(private::Internal) && vertex_bufs.$field_name.is_none() {
                    panic!(
                        "Vertex buffer {} is not given when its required",
                        stringify!($fieldname)
                    );
                } else if !$name::is_something(private::Internal)
                    && vertex_bufs.$field_name.is_some()
                {
                    panic!(
                        "Vertex buffer {} is given when its not required",
                        stringify!($fieldname)
                    );
                }

                if let Some(buf) = vertex_bufs.$field_name {
                    // Bind the buffer if needed
                    render_pass.set_vertex_buffer($idx_var, buf.slice(..));
                    $idx_var += 1;
                } else {
                    // Else dont need to bind the buffer
                }
            };
        }

        render_pass.set_pipeline(&self.pipeline);

        // Attach all vertex buffers
        let mut index = 0;
        check_vertex_buf!(Vertex0, buf0, index);
        check_vertex_buf!(Vertex1, buf1, index);
        check_vertex_buf!(Vertex2, buf2, index);
        check_vertex_buf!(Vertex3, buf3, index);
        check_vertex_buf!(Vertex4, buf4, index);
        check_vertex_buf!(Vertex5, buf5, index);
        check_vertex_buf!(Vertex6, buf6, index);
        check_vertex_buf!(Vertex7, buf7, index);

        assert_eq!(
            extra_vertex_bufs.len(),
            self.extra_bufs_count,
            "There more extra buffer given than pipeline constructed with"
        );

        for buf in extra_vertex_bufs.iter() {
            render_pass.set_vertex_buffer(index, buf.slice(..));
            index += 1;
        }

        // Then index buffer
        if Index::is_something(private::Internal) {
            let slice =
                index_buf_slice.expect("Pipeline requires an index buffer, but none was given");
            render_pass.set_index_buffer(slice, Index::INDEX_FORMAT);
        } else if index_buf_slice.is_some() {
            panic!("Pipeline does not require an index buffer, but Some was given");
        }

        // Finally perform the draw
        if Index::is_something(private::Internal) {
            let base_vertex =
                base_vertex.expect("Indexed rendering performed but base vertex is not given");

            // Indexed rendering
            render_pass.draw_indexed(vertices_range, base_vertex, instance_range);
        } else {
            if base_vertex.is_some() {
                panic!("Non indexed rendering performed but base vertex is given");
            }

            // Non indexed rendering
            render_pass.draw(vertices_range, instance_range);
        }
    }
}

// Nonindexed rendering
impl<
    Vertex0: HasVertexBufferLayout,
    Vertex1: HasVertexBufferLayout,
    Vertex2: HasVertexBufferLayout,
    Vertex3: HasVertexBufferLayout,
    Vertex4: HasVertexBufferLayout,
    Vertex5: HasVertexBufferLayout,
    Vertex6: HasVertexBufferLayout,
    Vertex7: HasVertexBufferLayout,
> Pipeline<(), Vertex0, Vertex1, Vertex2, Vertex3, Vertex4, Vertex5, Vertex6, Vertex7>
{
    #[expect(unused)]
    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass,
        vertex_bufs: &VertexBufs<
            Vertex0,
            Vertex1,
            Vertex2,
            Vertex3,
            Vertex4,
            Vertex5,
            Vertex6,
            Vertex7,
        >,

        // Same as one "vertices" parameter of wgpu::RenderPass::draw
        vertices: Range<u32>,
        instances: Range<u32>,

        // Additional buffer that won't be typechecked
        extra_vertex_bufs: &[&wgpu::Buffer],
    ) {
        self.render_impl(
            render_pass,
            vertex_bufs,
            None,
            vertices,
            instances,
            None,
            extra_vertex_bufs,
        );
    }
}

// Indexed rendering for 16-bit indices
impl<
    Vertex0: HasVertexBufferLayout,
    Vertex1: HasVertexBufferLayout,
    Vertex2: HasVertexBufferLayout,
    Vertex3: HasVertexBufferLayout,
    Vertex4: HasVertexBufferLayout,
    Vertex5: HasVertexBufferLayout,
    Vertex6: HasVertexBufferLayout,
    Vertex7: HasVertexBufferLayout,
> Pipeline<u16, Vertex0, Vertex1, Vertex2, Vertex3, Vertex4, Vertex5, Vertex6, Vertex7>
{
    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass,
        vertex_bufs: &VertexBufs<
            Vertex0,
            Vertex1,
            Vertex2,
            Vertex3,
            Vertex4,
            Vertex5,
            Vertex6,
            Vertex7,
        >,
        index_buf: &VecBuf<u16>,

        // There are the same parameter from wgpu::RenderPass::draw_indexed
        base_vertex: i32,
        indices: Range<u32>,
        instances: Range<u32>,

        // Additional buffer that won't be typechecked
        extra_vertex_bufs: &[&wgpu::Buffer],
    ) {
        self.render_impl(
            render_pass,
            vertex_bufs,
            Some(index_buf.slice(..)),
            indices,
            instances,
            Some(base_vertex),
            extra_vertex_bufs,
        );
    }
}

// Indexed rendering for 32-bit indices
impl<
    Vertex0: HasVertexBufferLayout,
    Vertex1: HasVertexBufferLayout,
    Vertex2: HasVertexBufferLayout,
    Vertex3: HasVertexBufferLayout,
    Vertex4: HasVertexBufferLayout,
    Vertex5: HasVertexBufferLayout,
    Vertex6: HasVertexBufferLayout,
    Vertex7: HasVertexBufferLayout,
> Pipeline<u32, Vertex0, Vertex1, Vertex2, Vertex3, Vertex4, Vertex5, Vertex6, Vertex7>
{
    #[expect(unused)]
    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass,
        vertex_bufs: &VertexBufs<
            Vertex0,
            Vertex1,
            Vertex2,
            Vertex3,
            Vertex4,
            Vertex5,
            Vertex6,
            Vertex7,
        >,
        index_buf: &VecBuf<u32>,

        // There are the same parameter from wgpu::RenderPass::draw_indexed
        base_vertex: i32,
        indices: Range<u32>,
        instances: Range<u32>,

        // Additional buffer that won't be typechecked
        extra_vertex_bufs: &[&wgpu::Buffer],
    ) {
        self.render_impl(
            render_pass,
            vertex_bufs,
            Some(index_buf.slice(..)),
            indices,
            instances,
            Some(base_vertex),
            extra_vertex_bufs,
        );
    }
}

mod private {
    pub struct Internal;
    pub trait Sealed {}
}

pub trait HasVertexBufferLayout: bytemuck::Pod + bytemuck::Zeroable + Clone + Copy {
    const LAYOUT: wgpu::VertexBufferLayout<'static>;

    // Lets pipeline know if a Vertex0, Vertex1, etc is not
    // present using () placeholder for none
    fn is_something(_internal: private::Internal) -> bool {
        true
    }
}

impl HasVertexBufferLayout for () {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![],
    };

    fn is_something(_internal: private::Internal) -> bool {
        false
    }
}

pub trait HasIndexFormat: private::Sealed {
    const INDEX_FORMAT: wgpu::IndexFormat;

    fn is_something(_internal: private::Internal) -> bool {
        true
    }
}

impl private::Sealed for u16 {}
impl HasIndexFormat for u16 {
    const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;
}

impl private::Sealed for u32 {}
impl HasIndexFormat for u32 {
    const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint32;
}

impl private::Sealed for () {}
impl HasIndexFormat for () {
    const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;

    fn is_something(_internal: private::Internal) -> bool {
        false
    }
}
