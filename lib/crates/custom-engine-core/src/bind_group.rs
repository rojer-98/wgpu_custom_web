pub mod entry;
pub mod layout;

use derive_more::{Deref, DerefMut};
use log::debug;

use crate::{buffer::Buffer, errors::CoreError, traits::Builder};

#[derive(Debug, Deref, DerefMut)]
pub struct BindGroup {
    pub id: usize,
    pub binding: u32,

    #[deref]
    #[deref_mut]
    inner_bg: wgpu::BindGroup,
}

pub struct BindGroupBuilder<'a> {
    id: Option<usize>,
    label: Option<&'a str>,
    entries: Option<Vec<wgpu::BindGroupEntry<'a>>>,
    layout: Option<&'a wgpu::BindGroupLayout>,
    binding: Option<u32>,

    device: &'a wgpu::Device,
}

impl<'a> Builder<'a> for BindGroupBuilder<'a> {
    type Final = BindGroup;

    fn new(device: &'a wgpu::Device) -> Self
    where
        Self: Sized,
    {
        Self {
            device,
            entries: None,
            layout: None,
            binding: None,
            id: None,
            label: None,
        }
    }

    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            device,
            entries: None,
            binding: None,
            layout: None,
            id: Some(id),
            label: None,
        }
    }

    fn build(self) -> Result<Self::Final, CoreError>
    where
        Self: Sized,
    {
        let id = self.id.unwrap_or_default();
        let bind_group_name = format!("Bind group: {id}");

        let binding = self
            .binding
            .ok_or(CoreError::EmptyBinding(bind_group_name.clone()))?;
        let label = self.label.unwrap_or(&bind_group_name);
        let layout = self
            .layout
            .ok_or(CoreError::EmptyLayout(label.to_string()))?;
        let entries = self
            .entries
            .ok_or(CoreError::EmptyEntries(label.to_string()))?;

        debug!(
            "
Build `{label}`:
    Entries: {entries:#?},
    Layout: {layout:?},"
        );

        let inner_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            label: Some(label),
            entries: entries.as_slice(),
        });

        Ok(BindGroup {
            id,
            inner_bg,
            binding,
        })
    }
}

impl<'a> BindGroupBuilder<'a> {
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn entries_buffer(mut self, buffer: &'a Buffer) -> Self {
        self.entries
            .get_or_insert(vec![])
            .push(wgpu::BindGroupEntry {
                binding: buffer.binding,
                resource: buffer.as_entire_binding(),
            });
        self
    }

    pub fn entries_buffers(mut self, buffers: Vec<&'a Buffer>) -> Self {
        let entries = self.entries.get_or_insert(vec![]);

        buffers.iter().for_each(|buffer| {
            entries.push(wgpu::BindGroupEntry {
                binding: buffer.binding,
                resource: buffer.as_entire_binding(),
            })
        });

        self
    }

    pub fn entries_sampler(mut self, binding: u32, sampler: &'a wgpu::Sampler) -> Self {
        self.entries
            .get_or_insert(vec![])
            .push(wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::Sampler(sampler),
            });

        self
    }

    pub fn entries_views(mut self, views: Vec<(u32, &'a wgpu::TextureView)>) -> Self {
        let entries = self.entries.get_or_insert(vec![]);

        views.iter().for_each(|(binding, view)| {
            entries.push(wgpu::BindGroupEntry {
                binding: *binding,
                resource: wgpu::BindingResource::TextureView(view),
            })
        });

        self
    }

    pub fn entries_view(mut self, binding: u32, view: &'a wgpu::TextureView) -> Self {
        self.entries
            .get_or_insert(vec![])
            .push(wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::TextureView(view),
            });

        self
    }

    pub fn layout(mut self, layout: &'a wgpu::BindGroupLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    pub fn binding(mut self, binding: u32) -> Self {
        self.binding = Some(binding);
        self
    }
}
