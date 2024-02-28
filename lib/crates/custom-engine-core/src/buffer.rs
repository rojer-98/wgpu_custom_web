use derive_more::{Deref, DerefMut};
use flume::bounded;
use log::{debug, error};

use crate::{errors::CoreError, traits::Builder};

#[derive(Debug, Deref, DerefMut)]
pub struct Buffer {
    pub id: usize,
    pub binding: u32,
    pub usage: wgpu::BufferUsages,

    #[deref]
    #[deref_mut]
    inner_buffer: wgpu::Buffer,
}

impl Buffer {
    fn read_buffer(&self) -> Vec<u8> {
        let buffer_slice = self.inner_buffer.slice(..);

        let view = buffer_slice.get_mapped_range().to_vec();
        self.inner_buffer.unmap();

        view
    }

    #[allow(dead_code)]
    fn write_buffer<T: bytemuck::Pod + bytemuck::Zeroable>(&self, data: &[T]) {
        let data: &[u8] = bytemuck::cast_slice(data);
        let buffer_slice = self.inner_buffer.slice(..);

        let mut view = buffer_slice.get_mapped_range_mut();
        view.copy_from_slice(data);

        self.inner_buffer.unmap();
    }

    pub(crate) async fn read_buffer_async(
        &self,
        device: &wgpu::Device,
    ) -> Result<Vec<u8>, CoreError> {
        let (tx, rx) = bounded(1);
        let buffer_slice = self.inner_buffer.slice(..);

        buffer_slice.map_async(wgpu::MapMode::Read, move |r| {
            if let Err(e) = tx.send(r) {
                error!("Buffer slice, map async error: {e}");
            }
        });

        device.poll(wgpu::Maintain::Wait);
        rx.recv_async().await??;

        Ok(self.read_buffer())
    }

    #[allow(dead_code)]
    pub(crate) async fn write_buffer_async<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        device: &wgpu::Device,
        data: &[T],
    ) -> Result<(), CoreError> {
        let (tx, rx) = bounded(1);
        let buffer_slice = self.inner_buffer.slice(..);

        buffer_slice.map_async(wgpu::MapMode::Write, move |r| {
            if let Err(e) = tx.send(r) {
                error!("Buffer slice, map async error: {e}");
            }
        });

        device.poll(wgpu::Maintain::Wait);
        rx.recv_async().await??;

        Ok(self.write_buffer(data))
    }
}

pub struct BufferBuilder<'a, T: bytemuck::Pod + bytemuck::Zeroable> {
    id: Option<usize>,
    label: Option<&'a str>,
    data: Option<&'a [T]>,
    usage: wgpu::BufferUsages,
    binding: u32,
    mapped_at_creation: bool,
    size: Option<u64>,

    device: &'a wgpu::Device,
}

impl<'a, T: bytemuck::Pod + bytemuck::Zeroable> Builder<'a> for BufferBuilder<'a, T> {
    type Final = Buffer;

    fn new(device: &'a wgpu::Device) -> Self {
        Self {
            id: Some(0),
            device,
            usage: wgpu::BufferUsages::VERTEX,
            binding: 0,
            data: None,
            label: None,
            size: None,
            mapped_at_creation: false,
        }
    }

    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self {
        Self {
            id: Some(id),
            device,
            usage: wgpu::BufferUsages::VERTEX,
            binding: 0,
            data: None,
            label: None,
            size: None,
            mapped_at_creation: false,
        }
    }

    fn build(self) -> Result<Self::Final, CoreError> {
        use wgpu::util::DeviceExt;

        let id = self.id.unwrap_or_default();
        let buffer_name = format!("Buffer: {id}");

        let label = self.label.unwrap_or(&buffer_name);
        let usage = self.usage;
        let binding = self.binding;
        let mapped_at_creation = self.mapped_at_creation;
        let size = self.size.unwrap_or_default();

        let inner_buffer = if let Some(d) = self.data {
            let mut contents = bytemuck::cast_slice(d).to_vec();

            if size != 0 {
                contents.resize(size as _, 0)
            }

            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents: &contents,
                    usage,
                })
        } else {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                usage,
                mapped_at_creation,
                size,
            })
        };

        debug!(
            "
Build `{label}`:
    Usage: {usage:?},
    Binding: {binding},
    Mapped at creation: {mapped_at_creation},
    Size: {size},
            "
        );

        Ok(Buffer {
            id,
            inner_buffer,
            binding,
            usage,
        })
    }
}

impl<'a, T: bytemuck::Pod + bytemuck::Zeroable> BufferBuilder<'a, T> {
    pub fn data(mut self, data: &'a [T]) -> Self {
        self.data = Some(data);
        self
    }

    pub fn usage(mut self, usage: wgpu::BufferUsages) -> Self {
        self.usage = usage;
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn binding(mut self, binding: u32) -> Self {
        self.binding = binding;
        self
    }

    pub fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn mapped_at_creation(mut self, mapped_at_creation: bool) -> Self {
        self.mapped_at_creation = mapped_at_creation;
        self
    }
}
