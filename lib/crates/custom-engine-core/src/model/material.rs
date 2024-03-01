use log::debug;

use crate::{
    bind_group::{BindGroup, BindGroupBuilder},
    errors::CoreError,
    texture::{RenderTexture, RenderTextureBuilder, TextureKind},
    traits::Builder,
};

#[derive(Debug)]
pub struct Material {
    pub id: usize,
    pub name: String,

    bind_group: BindGroup,

    diffuse_texture: RenderTexture,
    normal_texture: Option<RenderTexture>,
}

impl Material {
    pub fn store_textures_to_memory(&self, queue: &wgpu::Queue) {
        self.diffuse_texture.store_to_memory(queue);
        if let Some(n_t) = self.normal_texture.as_ref() {
            n_t.store_to_memory(queue);
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

pub struct MaterialBuilder<'a> {
    id: Option<usize>,
    name: Option<&'a str>,
    layout: Option<&'a wgpu::BindGroupLayout>,

    diffuse_view_binding: Option<u32>,
    diffuse_sampler_binding: Option<u32>,
    diffuse_texture_data: Option<&'a [u8]>,

    normal_view_binding: Option<u32>,
    normal_sampler_binding: Option<u32>,
    normal_texture_data: Option<&'a [u8]>,

    device: &'a wgpu::Device,
}

impl<'a> Builder<'a> for MaterialBuilder<'a> {
    type Final = Material;

    fn new(device: &'a wgpu::Device) -> Self
    where
        Self: Sized,
    {
        Self {
            name: None,
            id: None,
            normal_view_binding: None,
            normal_sampler_binding: None,
            normal_texture_data: None,
            diffuse_view_binding: None,
            diffuse_sampler_binding: None,
            diffuse_texture_data: None,
            layout: None,
            device,
        }
    }

    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            name: None,
            normal_view_binding: None,
            normal_sampler_binding: None,
            normal_texture_data: None,
            diffuse_view_binding: None,
            diffuse_sampler_binding: None,
            diffuse_texture_data: None,
            id: Some(id),
            layout: None,
            device,
        }
    }

    fn build(self) -> Result<Self::Final, CoreError>
    where
        Self: Sized,
    {
        let id = self.id.unwrap_or_default();
        let material_name = format!("Material: {id}");

        let name = self.name.unwrap_or(&material_name);

        let diffuse_texture_data = self
            .diffuse_texture_data
            .ok_or(CoreError::EmptyDiffuseTexture(name.to_string()))?;
        let diffuse_texture = RenderTextureBuilder::new(self.device)
            .label(&format!("Diffuse texture: {name}"))
            .bytes(&diffuse_texture_data)
            .build()?;
        let diff_view = &diffuse_texture.view;
        let diff_sampler =
            diffuse_texture
                .sampler
                .as_ref()
                .ok_or(CoreError::EmptyTextureSampler(format!(
                    "{}",
                    diffuse_texture.id
                )))?;
        let diffuse_view_binding = self
            .diffuse_view_binding
            .ok_or(CoreError::EmptyBinding(name.to_string()))?;
        let diffuse_sampler_binding = self
            .diffuse_sampler_binding
            .ok_or(CoreError::EmptyBinding(name.to_string()))?;

        let name = name.to_string();
        let layout = self
            .layout
            .ok_or(CoreError::EmptyLayout(name.to_string()))?;

        let bind_group_name = format!("Bind group of: {name}");
        let bind_group = BindGroupBuilder::new(self.device)
            .label(&bind_group_name)
            .layout(&layout)
            .binding(0)
            .entries_view(diffuse_view_binding, diff_view)
            .entries_sampler(diffuse_sampler_binding, diff_sampler);

        let (bind_group, normal_texture) = if let Some(data) = self.normal_texture_data {
            let normal_texture = RenderTextureBuilder::new(self.device)
                .label(&format!("Normal texture: {name}"))
                .bytes(&data)
                .format(TextureKind::NormalMap)
                .build()?;

            let norm_view = &normal_texture.view;
            let norm_sampler =
                normal_texture
                    .sampler
                    .as_ref()
                    .ok_or(CoreError::EmptyTextureSampler(format!(
                        "{}",
                        normal_texture.id
                    )))?;
            let normal_view_binding = self
                .normal_view_binding
                .ok_or(CoreError::EmptyBinding(name.to_string()))?;
            let normal_sampler_binding = self
                .normal_sampler_binding
                .ok_or(CoreError::EmptyBinding(name.to_string()))?;

            (
                bind_group
                    .entries_view(normal_view_binding, norm_view)
                    .entries_sampler(normal_sampler_binding, norm_sampler)
                    .build()?,
                Some(normal_texture),
            )
        } else {
            (bind_group.build()?, None)
        };

        debug!(
            "
Build `{name}`:
    Normal texture: {normal_texture:#?},
    Diffuse texture: {diffuse_texture:#?},
    Bind group: {bind_group:#?},
            "
        );

        Ok(Material {
            id,
            name,
            diffuse_texture,
            normal_texture,
            bind_group,
        })
    }
}

impl<'a> MaterialBuilder<'a> {
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn diffuse_texture_data(mut self, diffuse_texture_data: Option<&'a [u8]>) -> Self {
        self.diffuse_texture_data = diffuse_texture_data.into();
        self
    }

    pub fn normal_texture_data(mut self, normal_texture_data: Option<&'a [u8]>) -> Self {
        self.normal_texture_data = normal_texture_data.into();
        self
    }

    pub fn layout(mut self, layout: &'a wgpu::BindGroupLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    pub fn normal_sampler_binding(mut self, normal_sampler_binding: u32) -> Self {
        self.normal_sampler_binding = Some(normal_sampler_binding);
        self
    }

    pub fn normal_view_binding(mut self, normal_view_binding: u32) -> Self {
        self.normal_view_binding = Some(normal_view_binding);
        self
    }

    pub fn diffuse_sampler_binding(mut self, diffuse_sampler_binding: u32) -> Self {
        self.diffuse_sampler_binding = Some(diffuse_sampler_binding);
        self
    }

    pub fn diffuse_view_binding(mut self, diffuse_view_binding: u32) -> Self {
        self.diffuse_view_binding = Some(diffuse_view_binding);
        self
    }
}
