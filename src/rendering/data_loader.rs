use std::{marker::PhantomData, num::NonZero, ops::RangeBounds, ptr::NonNull};

use crate::rendering::util;

// This structure goal is loading data into GPU
// whether it be textures or buffers
pub struct DataLoader {
    device: wgpu::Device,

    // NOTE: wgpu currently only have single queue which is shared by all
    // cloned. This is here to seperate concerns hopefully in future this
    // can be swapped with TransferQueue or something similar
    upload_queue: wgpu::Queue,
}

pub struct WriteBufferView<'a> {
    view: Option<wgpu::QueueWriteBufferView>,
    _phantom: PhantomData<&'a u32>,
}

impl WriteBufferView<'_> {
    pub fn len(&self) -> usize {
        self.view.as_ref().map(|x| x.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.view.as_ref().map(|x| x.is_empty()).unwrap_or(true)
    }

    pub fn slice<'a, R: RangeBounds<usize>>(&'a mut self, bounds: R) -> wgpu::WriteOnly<'a, [u8]> {
        if let Some(x) = self.view.as_mut() {
            return x.slice(bounds);
        } else {
            if !bounds.is_empty() {
                panic!("Attempting to index WriteBufferView that is empty with non empty range");
            }

            // SAFETY: Its empty slice, so use dangling, it must not be accesible
            unsafe { wgpu::WriteOnly::new(NonNull::slice_from_raw_parts(NonNull::dangling(), 0)) }
        }
    }

    pub fn copy_from_slice(&mut self, data: &[u8]) {
        if let Some(x) = self.view.as_mut() {
            x.copy_from_slice(data)
        }
    }
}

impl DataLoader {
    pub(super) fn new(device: wgpu::Device, upload_queue: wgpu::Queue) -> Self {
        Self {
            device,
            upload_queue,
        }
    }

    pub fn write_buffer(&self, buf: &wgpu::Buffer, offset: wgpu::BufferAddress, data: &[u8]) {
        self.upload_queue.write_buffer(buf, offset, data);
    }

    pub fn write_buffer_with<F, R>(
        &self,
        buf: &wgpu::Buffer,
        offset: wgpu::BufferAddress,
        size: wgpu::BufferAddress,
        func: F,
    ) -> R
    where
        F: FnOnce(&mut WriteBufferView<'_>) -> R,
    {
        let mut view = WriteBufferView {
            view: NonZero::new(size).map(|size| {
                self.upload_queue
                    .write_buffer_with(buf, offset, size)
                    .expect("Foxie has 0 clue why this return None :<")
            }),
            _phantom: PhantomData,
        };

        func(&mut view)
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn copy_from_buffer_to_buffer(
        &self,
        src: &wgpu::Buffer,
        dest: &wgpu::Buffer,
        offset_src: wgpu::BufferAddress,
        offset_dest: wgpu::BufferAddress,
        size: wgpu::BufferAddress,
    ) {
        if size == 0 {
            return;
        }

        if !offset_src.is_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT)
            || !offset_dest.is_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT)
            || !size.is_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT)
        {
            panic!(
                "Source offset or destination offset and size is not multiple of wgpu::COPY_BUFFER_ALIGNMENT which is {}",
                wgpu::COPY_BUFFER_ALIGNMENT
            );
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("GPU to GPU buffer copy from DataLoader"),
            });
        encoder.copy_buffer_to_buffer(src, offset_src, dest, offset_dest, Some(size));

        util::wait_device(&self.device, self.upload_queue.submit([encoder.finish()]));
    }

    pub fn load_texture(&self, texture: image::DynamicImage) -> wgpu::Texture {
        let texture_raw = texture.to_rgba8();
        let width = texture_raw.dimensions().0;
        let height = texture_raw.dimensions().1;
        let size = wgpu::Extent3d {
            depth_or_array_layers: 1,
            width,
            height,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            view_formats: &[],
            mip_level_count: 1,
            sample_count: 1,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: None,
        });

        self.upload_queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                aspect: wgpu::TextureAspect::All,
                origin: wgpu::Origin3d::ZERO,
            },
            &texture_raw,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        texture
    }
}
