use std::{cmp, marker::PhantomData, mem, ops::RangeBounds};

use crate::rendering::data_loader::DataLoader;

// Its a typed buffer for wgpu::Buffer
// Its act moreless like Vec but for GPU buffer
// with familiar function like push, copy_from_slice
// set_len, etc then extend_from_slice and so on
//
// Its also read-only and NOTE data_loader must be
// built from same Device given at construction
pub struct VecBuf<T: Copy + bytemuck::Pod> {
    buf: wgpu::Buffer,
    device: wgpu::Device,
    len: usize,
    _phantom: PhantomData<T>,
}

// Asking GPU to copy buffer to buffer kind of
// expensive so lets start with large initial
// size
pub const DEFAULT_INITIAL_CAPACITY: usize = 64;

pub enum BufferKind {
    Index,
    Vertex,
    Uniform,
    Storage,
    Indirect,
}

impl<T: Copy + bytemuck::Pod> VecBuf<T> {
    pub fn new(device: wgpu::Device, kind: BufferKind) -> Self {
        Self::new_with_initial_capacity(device, kind, DEFAULT_INITIAL_CAPACITY)
    }

    pub fn new_with_initial_capacity(
        device: wgpu::Device,
        kind: BufferKind,
        mut capacity: usize,
    ) -> Self {
        let mut usage = wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;

        match kind {
            BufferKind::Index => usage |= wgpu::BufferUsages::INDEX,
            BufferKind::Vertex => usage |= wgpu::BufferUsages::VERTEX,
            BufferKind::Uniform => usage |= wgpu::BufferUsages::UNIFORM,
            BufferKind::Storage => usage |= wgpu::BufferUsages::STORAGE,
            BufferKind::Indirect => usage |= wgpu::BufferUsages::INDIRECT,
        }

        // Turn capacity into power of two
        if !capacity.is_power_of_two() {
            capacity = capacity.next_power_of_two();
        }

        Self {
            len: 0,
            buf: device.create_buffer(&wgpu::BufferDescriptor {
                size: wgpu::BufferAddress::try_from(capacity * size_of::<T>())
                    .expect("capacity is too large than 64-bit limit"),
                mapped_at_creation: false,
                label: None,
                usage,
            }),
            _phantom: PhantomData,
            device,
        }
    }

    pub fn capacity(&self) -> usize {
        usize::try_from(self.buf.size()).expect("buffer somehow larger than usize's limit")
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn copy_from_slice(&mut self, data_loader: &DataLoader, data: &[T]) {
        assert_eq!(
            self.len,
            data.len(),
            "Attempt to copy data differing length to buffer, both length of destination and source must be same"
        );
        self.check_capacity(data_loader, self.len() + data.len());
        data_loader.write_buffer(&self.buf, 0, bytemuck::cast_slice(data));
    }

    pub fn set(&mut self, index: usize, data_loader: &DataLoader, data: &T) {
        assert!(index < self.len, "Attempt to write over the bound");
        data_loader.write_buffer(
            &self.buf,
            u64::try_from(index * size_of::<T>()).unwrap(),
            bytemuck::bytes_of(data),
        );
    }

    pub fn extend_from_slice(&mut self, data_loader: &DataLoader, data: &[T]) {
        self.check_capacity(data_loader, self.len() + data.len());

        let start_offset = self.len * size_of::<T>();
        self.len += data.len();

        data_loader.write_buffer(
            &self.buf,
            wgpu::BufferAddress::try_from(start_offset).unwrap(),
            bytemuck::cast_slice(data),
        );
    }

    fn check_capacity(&mut self, data_loader: &DataLoader, min_capacity: usize) {
        if min_capacity <= self.capacity() {
            // No expansion necessary
            return;
        }

        // Extend capacity as necessary
        self.resize_capacity_possibly_drop_data(data_loader, min_capacity);
    }

    // Compacts the buffer, so its smallest can fit
    pub fn compact(&mut self, data_loader: &DataLoader) {
        self.resize_capacity_possibly_drop_data(data_loader, self.len());
    }

    // This implicitly also extend buffer if needed
    // the bit patttern in the new space is valid (due bytemuck::Pod)
    // but meaningless/unknown and should be written with meaningful
    // value
    //
    // When shrinking, the buffer itself won't compact and the data after
    // the len elements, is now considered to be unknown/uninitialized but
    // valid pattern.
    pub fn resize(&mut self, data_loader: &DataLoader, new_len: usize) {
        self.check_capacity(data_loader, new_len);
        self.len = new_len;
    }

    // NOTE: Drop code will never run on this
    // as T is Copy which means drop code cant exist
    fn resize_capacity_possibly_drop_data(
        &mut self,
        data_loader: &DataLoader,
        mut new_capacity: usize,
    ) {
        if !new_capacity.is_power_of_two() {
            new_capacity = new_capacity.next_power_of_two();
        }

        let resized = self.device.create_buffer(&wgpu::BufferDescriptor {
            size: wgpu::BufferAddress::try_from(new_capacity * size_of::<T>())
                .expect("capacity is too large than 64-bit limit"),
            usage: self.buf.usage(),
            mapped_at_creation: false,
            label: None,
        });

        let old = mem::replace(&mut self.buf, resized);

        // Copy old to new
        data_loader.copy_from_buffer_to_buffer(
            &old,
            &self.buf,
            0,
            0,
            cmp::min(old.size(), self.buf.size()),
        );

        old.destroy();
    }

    // NOTE: DO NOT modify the buffer
    // it is read-only. If the buffer is
    // readable. Regardlesss if its writeable
    // or not
    //
    // The slice is in temr of T units
    pub fn slice<'a, S: RangeBounds<usize>>(&'a self, bounds: S) -> wgpu::BufferSlice<'a> {
        let start = match bounds.start_bound() {
            std::ops::Bound::Unbounded => 0,
            std::ops::Bound::Excluded(&idx) => idx + 1,
            std::ops::Bound::Included(&idx) => idx,
        };

        let end = match bounds.end_bound() {
            std::ops::Bound::Unbounded => self.len,
            std::ops::Bound::Excluded(&idx) => idx - 1,
            std::ops::Bound::Included(&idx) => idx,
        };

        self.buf.slice(
            (start * size_of::<T>()) as wgpu::BufferAddress
                ..(end * size_of::<T>()) as wgpu::BufferAddress,
        )
    }

    pub fn new_from_slice(
        device: wgpu::Device,
        data_loader: &DataLoader,
        kind: BufferKind,
        data: &[T],
    ) -> Self {
        let mut buf = Self::new_with_initial_capacity(device, kind, data.len());
        buf.extend_from_slice(data_loader, data);
        buf
    }

    pub fn as_binding<'a>(&'a self) -> wgpu::BindingResource<'a> {
        self.buf.as_entire_binding()
    }
}
