use std::{borrow::Cow, marker::PhantomData, ops::RangeBounds};

use bytemuck::Pod;
use log::debug;
use wgpu::BufferAddress;

#[derive(Debug)]
pub(crate) struct GpuVec<T: Pod> {
    buffer: wgpu::Buffer,
    /// Allocated capacity, in ELEMENTS (like Vec, unlike wgpu's byte sizes).
    capacity: u64,
    /// Elements made valid by the most recent `write`.
    len: u32,
    usage: wgpu::BufferUsages,
    label: Option<Cow<'static, str>>,
    _marker: PhantomData<T>,
}

impl<T: Pod> GpuVec<T> {
    pub(crate) fn new(
        device: &wgpu::Device,
        label: Option<impl Into<Cow<'static, str>>>,
        usage: wgpu::BufferUsages,
        initial_capacity: u64,
    ) -> Self {
        let usage = usage | wgpu::BufferUsages::COPY_DST;
        let label = label.map(Into::into);

        Self {
            buffer: Self::alloc(device, label.as_deref(), usage, initial_capacity),
            capacity: initial_capacity,
            len: 0,
            usage,
            label,
            _marker: PhantomData,
        }
    }

    fn alloc(
        device: &wgpu::Device,
        label: Option<&str>,
        usage: wgpu::BufferUsages,
        capacity: u64,
    ) -> wgpu::Buffer {
        device.create_buffer(&wgpu::wgt::BufferDescriptor {
            label,
            size: capacity * size_of::<T>() as u64,
            usage,
            mapped_at_creation: false,
        })
    }

    /// Replace's the buffer's contents with `data`, growing if needed.
    pub(crate) fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[T]) {
        let needed = data.len() as u64;
        if needed > self.capacity {
            let new_capacity = needed.next_power_of_two();
            debug!(
                "GpuVec '{:?}' grows {} -> {new_capacity} elements",
                self.label.as_deref(),
                self.capacity
            );

            self.buffer = Self::alloc(device, self.label.as_deref(), self.usage, new_capacity);
            self.capacity = new_capacity;
        }

        if !data.is_empty() {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
        }

        self.len = u32::try_from(data.len()).expect("GpuVec length exceeds u32");
    }

    pub(crate) fn len(&self) -> u32 {
        self.len
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn slice<S>(&self, bounds: S) -> wgpu::BufferSlice<'_>
    where
        S: RangeBounds<BufferAddress>,
    {
        self.buffer.slice(bounds)
    }
}
