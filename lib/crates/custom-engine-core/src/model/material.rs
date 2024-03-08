use log::debug;

use crate::{
    bind_group::{BindGroup, BindGroupBuilder},
    errors::CoreError,
    texture::{RenderTexture, RenderTextureBuilder},
    traits::Builder,
};

#[derive(Debug)]
pub struct MaterialTextureParams<'a> {
    pub view_binding: Option<u32>,
    pub sampler_binding: Option<u32>,
    pub texture_data: Option<&'a [u8]>,
    pub format: wgpu::TextureFormat,
}

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

    diffuse: Option<MaterialTextureParams<'a>>,
    normal: Option<MaterialTextureParams<'a>>,

    material_binding: u32,

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
            normal: None,
            diffuse: None,
            layout: None,
            material_binding: 0,
            device,
        }
    }

    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            name: None,
            id: Some(id),
            normal: None,
            diffuse: None,
            layout: None,
            material_binding: 0,
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
        let material_binding = self.material_binding;

        let diffuse = self
            .diffuse
            .ok_or(CoreError::EmptyDiffuseTexture(name.to_string()))?;

        let diffuse_texture_data = diffuse
            .texture_data
            .ok_or(CoreError::EmptyDiffuseTexture(name.to_string()))?;
        let diffuse_texture = RenderTextureBuilder::new(self.device)
            .label(&format!("Diffuse texture: {name}"))
            .bytes(diffuse_texture_data)
            .format(diffuse.format)
            .usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST)
            .build()?;
        let diff_view = diffuse_texture.view();
        let diff_sampler = diffuse_texture.sampler()?;
        let diffuse_view_binding = diffuse
            .view_binding
            .ok_or(CoreError::EmptyBinding(name.to_string()))?;
        let diffuse_sampler_binding = diffuse
            .sampler_binding
            .ok_or(CoreError::EmptyBinding(name.to_string()))?;

        let name = name.to_string();
        let layout = self
            .layout
            .ok_or(CoreError::EmptyLayout(name.to_string()))?;

        let bind_group_name = format!("Bind group of: {name}");
        let bind_group = BindGroupBuilder::new(self.device)
            .label(&bind_group_name)
            .layout(layout)
            .binding(material_binding)
            .entries_view(diffuse_view_binding, diff_view)
            .entries_sampler(diffuse_sampler_binding, diff_sampler);

        let (bind_group, normal_texture) = if let Some(normal) = self.normal {
            let normal_texture_data = normal
                .texture_data
                .ok_or(CoreError::EmptyNormalTexture(name.to_string()))?;
            let normal_texture = RenderTextureBuilder::new(self.device)
                .label(&format!("Normal texture: {name}"))
                .bytes(&normal_texture_data)
                .format(normal.format)
                .usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST)
                .build()?;

            let norm_view = normal_texture.view();
            let norm_sampler = normal_texture.sampler()?;
            let normal_view_binding = normal
                .view_binding
                .ok_or(CoreError::EmptyBinding(name.to_string()))?;
            let normal_sampler_binding = normal
                .sampler_binding
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

    pub fn layout(mut self, layout: &'a wgpu::BindGroupLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    pub fn diffuse(mut self, mtp: MaterialTextureParams<'a>) -> Self {
        self.diffuse = Some(mtp);
        self
    }

    pub fn normal(mut self, mtp: MaterialTextureParams<'a>) -> Self {
        self.normal = Some(mtp);
        self
    }

    pub fn material_binding(mut self, binding: u32) -> Self {
        self.material_binding = binding;
        self
    }
}
