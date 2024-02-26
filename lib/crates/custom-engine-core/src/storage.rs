use std::collections::HashMap;

use log::debug;

use crate::{
    bind_group::{
        layout::{BindGroupLayout, BindGroupLayoutBuilder},
        BindGroup, BindGroupBuilder,
    },
    buffer::{Buffer, BufferBuilder},
    errors::CoreError,
    texture::{RenderTexture, RenderTextureBuilder},
    traits::Builder,
};

#[derive(Debug)]
pub struct Storages {
    pub id: usize,
    pub name: String,

    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,

    buffers: HashMap<String, Buffer>,
    textures: HashMap<String, RenderTexture>,
}

impl Storages {
    pub fn get_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn get_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn get_buffer(&self, name: &str) -> Option<&Buffer> {
        self.buffers.get(name)
    }

    pub fn get_texture(&self, name: &str) -> Option<&RenderTexture> {
        self.textures.get(name)
    }
}

#[derive(Debug)]
pub enum StorageKind {
    Buffer {
        read_only: bool,
    },
    Texture {
        size: u32,
        access: wgpu::StorageTextureAccess,
        format: wgpu::TextureFormat,
        view_dimension: wgpu::TextureViewDimension,
    },
}

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct StorageDescription<'a> {
    name: &'a str,
    binding: u32,
    visibility: wgpu::ShaderStages,
    kind: StorageKind,
    #[derivative(Debug = "ignore")]
    data: &'a [u8],
}

impl<'a> StorageDescription<'a> {
    pub fn new<T: bytemuck::Pod + bytemuck::Zeroable>(
        name: &'a str,
        binding: u32,
        visibility: wgpu::ShaderStages,
        kind: StorageKind,
        data: &'a [T],
    ) -> Self {
        Self {
            name,
            binding,
            visibility,
            kind,
            data: bytemuck::cast_slice(data),
        }
    }
}

#[derive(Debug)]
pub struct StoragesBuilder<'a> {
    id: Option<usize>,
    name: Option<&'a str>,
    entries: Option<Vec<StorageDescription<'a>>>,
    bind_group_binding: Option<u32>,

    device: &'a wgpu::Device,
}

impl<'a> Builder<'a> for StoragesBuilder<'a> {
    type Final = Storages;

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
        let uniform_name = format!("Storage: {id}");

        let name = self.name.unwrap_or(&uniform_name);
        let bind_group_binding = self.bind_group_binding.unwrap_or_default();
        let entries = self
            .entries
            .ok_or(CoreError::EmptyEntries(name.to_string()))?;

        let bgl_name = format!("Bind group layout of `{name}`");
        let mut bgl_builder = BindGroupLayoutBuilder::new(self.device).label(&bgl_name);

        let mut buffers = HashMap::new();
        let mut views = vec![];
        for entry in entries.into_iter() {
            let StorageDescription {
                name,
                binding,
                visibility,
                data,
                kind,
                ..
            } = entry;

            match kind {
                StorageKind::Buffer { read_only } => {
                    bgl_builder = bgl_builder.entries(wgpu::BindGroupLayoutEntry {
                        visibility,
                        binding,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only },
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
                            .usage(
                                wgpu::BufferUsages::STORAGE
                                    | wgpu::BufferUsages::COPY_DST
                                    | wgpu::BufferUsages::MAP_READ
                                    | wgpu::BufferUsages::MAP_WRITE,
                            )
                            .build()?,
                    );
                }
                StorageKind::Texture {
                    size,
                    format,
                    access,
                    view_dimension,
                } => {
                    bgl_builder = bgl_builder.entries(wgpu::BindGroupLayoutEntry {
                        visibility,
                        binding,
                        ty: wgpu::BindingType::StorageTexture {
                            access,
                            format,
                            view_dimension,
                        },
                        count: None,
                    });

                    let texture = RenderTextureBuilder::new(self.device)
                        .label(name)
                        .texture_size((size, size))
                        .is_normal_map(false)
                        .texture_view_desc(Default::default())
                        .texture_desc(wgpu::TextureDescriptor {
                            label: Some(name),
                            size: wgpu::Extent3d {
                                width: size,
                                height: size,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format,
                            usage: wgpu::TextureUsages::STORAGE_BINDING
                                | wgpu::TextureUsages::COPY_SRC,
                            view_formats: &[],
                        })
                        .build()?;

                    views.push((name.to_string(), binding, texture));
                }
            }
        }

        let bg_name = format!("Bind group of `{name}`");
        let bind_group_layout = bgl_builder.build()?;
        let bind_group = BindGroupBuilder::new(self.device)
            .label(&bg_name)
            .binding(bind_group_binding)
            .layout(&bind_group_layout)
            .entries_buffers(buffers.iter().map(|(_, b)| b).collect::<Vec<_>>())
            .entries_views(
                views
                    .iter()
                    .map(|(_, b, rt)| (*b, &rt.view))
                    .collect::<Vec<_>>(),
            )
            .build()?;
        let name = name.to_string();

        let textures = views
            .into_iter()
            .map(|(name, _, rt)| (name.to_string(), rt))
            .collect::<HashMap<_, _>>();
        debug!(
            "
Build `{name}`: 
    Buffers: {buffers:#?},
    Textures: {textures:#?},
    Bind Group Layout: {bind_group_layout:#?},
    Bind Group: {bind_group:#?},
    "
        );

        Ok(Storages {
            id,
            name,
            bind_group,
            bind_group_layout,
            buffers,
            textures,
        })
    }
}

impl<'a> StoragesBuilder<'a> {
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn entries(mut self, entry: StorageDescription<'a>) -> Self {
        self.entries.get_or_insert(vec![]).push(entry);
        self
    }

    pub fn bind_group_binding(mut self, bind_group_binding: u32) -> Self {
        self.bind_group_binding = Some(bind_group_binding);
        self
    }
}
