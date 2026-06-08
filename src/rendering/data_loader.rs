// This structure goal is loading data into GPU
// whether it be textures or buffers
pub struct DataLoader {
    device: wgpu::Device,

    // NOTE: wgpu currently only have single queue which is shared by all
    // cloned. This is here to seperate concerns hopefully in future this
    // can be swapped with TransferQueue or something similar
    upload_queue: wgpu::Queue,
}

impl DataLoader {
    pub(super) fn new(device: wgpu::Device, upload_queue: wgpu::Queue) -> Self {
        Self {
            device,
            upload_queue,
        }
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
