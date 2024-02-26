use std::collections::HashMap;

use log::debug;

use crate::{
    bind_group::{
        layout::{BindGroupLayout, BindGroupLayoutBuilder},
        BindGroup, BindGroupBuilder,
    },
    buffer::{Buffer, BufferBuilder},
    errors::CoreError,
    traits::Builder,
};

#[derive(Debug)]
pub struct Uniforms {
    pub id: usize,
    pub name: String,

    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
    buffers: HashMap<String, Buffer>,
}

impl Uniforms {
    pub fn get_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn get_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn get_buffer(&self, name: &str) -> Option<&Buffer> {
        self.buffers.get(name)
    }
}

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct UniformDescription<'a> {
    name: &'a str,
    binding: u32,
    visibility: wgpu::ShaderStages,
    #[derivative(Debug = "ignore")]
    data: &'a [u8],
}

impl<'a> UniformDescription<'a> {
    pub fn new<T: bytemuck::Pod + bytemuck::Zeroable>(
        name: &'a str,
        binding: u32,
        visibility: wgpu::ShaderStages,
        data: &'a [T],
    ) -> Self {
        Self {
            name,
            binding,
            visibility,
            data: bytemuck::cast_slice(data),
        }
    }
}

#[derive(Debug)]
pub struct UniformsBuilder<'a> {
    id: Option<usize>,
    name: Option<&'a str>,
    entries: Option<Vec<UniformDescription<'a>>>,
    bind_group_binding: Option<u32>,

    device: &'a wgpu::Device,
}

impl<'a> Builder<'a> for UniformsBuilder<'a> {
    type Final = Uniforms;

    fn new(device: &'a wgpu::Device) -> Self
    where
        Self: Sized,
    {
        Self {
            bind_group_binding: None,
            entries: None,
            name: None,
            id: None,
            device,
        }
    }

    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            bind_group_binding: None,
            entries: None,
            name: None,
            id: Some(id),
            device,
        }
    }

    fn build(self) -> Result<Self::Final, CoreError>
    where
        Self: Sized,
    {
        let id = self.id.unwrap_or_default();
        let uniform_name = format!("Uniform: {id}");

        let name = self.name.unwrap_or(&uniform_name);
        let bind_group_binding = self.bind_group_binding.unwrap_or_default();
        let entries = self
            .entries
            .ok_or(CoreError::EmptyEntries(name.to_string()))?;

        let bgl_name = format!("Bind group layout of `{name}`");
        let mut bgl_builder = BindGroupLayoutBuilder::new(self.device).label(&bgl_name);

        let mut buffers = HashMap::new();
        for entry in entries.into_iter() {
            let UniformDescription {
                name,
                binding,
                visibility,
                data,
                ..
            } = entry;

            bgl_builder = bgl_builder.entries(wgpu::BindGroupLayoutEntry {
                visibility,
                binding,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });

            buffers.insert(
                name.to_string(),
                BufferBuilder::new(self.device)
                    .label(&name)
                    .binding(binding)
                    .data(data)
                    .usage(wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST)
                    .build()?,
            );
        }

        let bg_name = format!("Bind group of `{name}`");
        let bind_group_layout = bgl_builder.build()?;
        let bind_group = BindGroupBuilder::new(self.device)
            .label(&bg_name)
            .binding(bind_group_binding)
            .layout(&bind_group_layout)
            .entries_buffers(buffers.iter().map(|(_, b)| b).collect::<Vec<_>>())
            .build()?;
        let name = name.to_string();

        debug!(
            "
Build `{name}`: 
    Buffers: {buffers:#?},
    Bind Group Layout: {bind_group_layout:#?},
    Bind Group: {bind_group:#?},
    "
        );

        Ok(Uniforms {
            id,
            name,
            bind_group,
            bind_group_layout,
            buffers,
        })
    }
}

impl<'a> UniformsBuilder<'a> {
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn entries(mut self, entry: UniformDescription<'a>) -> Self {
        self.entries.get_or_insert(vec![]).push(entry);
        self
    }

    pub fn bind_group_binding(mut self, bind_group_binding: u32) -> Self {
        self.bind_group_binding = Some(bind_group_binding);
        self
    }
}
