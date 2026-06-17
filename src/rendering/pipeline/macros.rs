macro_rules! vertex_buffer_layout {
    ($type:ty as $mode:ident => [ $($attributes:tt)* ]) => {
        impl $crate::rendering::pipeline::HasVertexBufferLayout for $type {
            const LAYOUT: wgpu::VertexBufferLayout<'static> =wgpu::VertexBufferLayout {
                array_stride: size_of::<Self>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::$mode,
                attributes: &wgpu::vertex_attr_array![ $($attributes)* ]
            };
        }
    }
}

pub(crate) use vertex_buffer_layout;
