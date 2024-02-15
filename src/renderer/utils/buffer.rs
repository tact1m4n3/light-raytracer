use std::marker::PhantomData;
use std::mem;

use wgpu::util::DeviceExt;

pub type UniformBuffer<T> = Buffer<T, Uniform>;
pub type StorageBuffer<T> = Buffer<T, Storage>;

pub struct Buffer<T: bytemuck::Pod, U> {
    inner: wgpu::Buffer,
    len: usize,
    _dtype: PhantomData<T>,
    _usage: PhantomData<U>,
}

impl<T: bytemuck::Pod, U> Buffer<T, U> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.inner.as_entire_binding()
    }

    pub fn write(&self, queue: &wgpu::Queue, data: &[T]) {
        debug_assert_eq!(self.len, data.len());
        queue.write_buffer(&self.inner, 0, bytemuck::cast_slice(data));
    }
}

impl<T: bytemuck::Pod> Buffer<T, Uniform> {
    pub fn new(device: &wgpu::Device, label: &str, len: usize) -> Self {
        let inner = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (len * mem::size_of::<T>()) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            inner,
            len,
            _dtype: PhantomData,
            _usage: PhantomData,
        }
    }

    pub fn new_with_data(device: &wgpu::Device, label: &str, data: &[T]) -> Self {
        let inner = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            inner,
            len: data.len(),
            _dtype: PhantomData,
            _usage: PhantomData,
        }
    }
}

impl<T: bytemuck::Pod> Buffer<T, Storage> {
    pub fn new(device: &wgpu::Device, label: &str, len: usize) -> Self {
        let inner = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (len * mem::size_of::<T>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            inner,
            len,
            _dtype: PhantomData,
            _usage: PhantomData,
        }
    }

    pub fn new_with_data(device: &wgpu::Device, label: &str, data: &[T]) -> Self {
        let inner = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            inner,
            len: data.len(),
            _dtype: PhantomData,
            _usage: PhantomData,
        }
    }
}

pub struct Uniform;
pub struct Storage;
