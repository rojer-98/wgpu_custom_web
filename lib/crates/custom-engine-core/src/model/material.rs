use log::debug;

use crate::{
    bind_group::{BindGroup, BindGroupBuilder},
    errors::CoreError,
    texture::{RenderTexture, RenderTextureBuilder},
    traits::Builder,
};

#[derive(Debug)]
pub struct MaterialTextureParams<'a> {
    pub view_binding: u32,
    pub sampler_binding: u32,
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
    occlusion_texture: Option<RenderTexture>,
    mr_texture: Option<RenderTexture>,
    emissive_texture: Option<RenderTexture>,
}

impl Material {
    pub fn store_textures_to_memory(&self, queue: &wgpu::Queue) {
        self.diffuse_texture.store_to_memory(queue);
        if let Some(n_t) = self.normal_texture.as_ref() {
            n_t.store_to_memory(queue);
        }
        if let Some(o_t) = self.occlusion_texture.as_ref() {
            o_t.store_to_memory(queue)
        }
        if let Some(mr_t) = self.mr_texture.as_ref() {
            mr_t.store_to_memory(queue)
        }
        if let Some(e_t) = self.emissive_texture.as_ref() {
            e_t.store_to_memory(queue)
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
    occlusion: Option<MaterialTextureParams<'a>>,
    mr: Option<MaterialTextureParams<'a>>,
    emissive: Option<MaterialTextureParams<'a>>,

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
            mr: None,
            emissive: None,
            occlusion: None,
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
            mr: None,
            emissive: None,
            occlusion: None,
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
            .as_ref()
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
        let diffuse_view_binding = diffuse.view_binding;
        let diffuse_sampler_binding = diffuse.sampler_binding;

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

        let mut normal_texture = None;
        let bind_group = if let Some(mtp) = self.normal {
            let texture_data = mtp
                .texture_data
                .ok_or(CoreError::EmptyNormalTexture(name.to_string()))?;
            normal_texture = Some(
                RenderTextureBuilder::new(&self.device)
                    .label(&format!("Texture: {name}"))
                    .bytes(&texture_data)
                    .format(mtp.format)
                    .usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST)
                    .build()?,
            );

            let view = normal_texture.as_ref().unwrap().view();
            let sampler = normal_texture.as_ref().unwrap().sampler()?;
            let view_binding = mtp.view_binding;
            let sampler_binding = mtp.sampler_binding;

            bind_group
                .entries_view(view_binding, view)
                .entries_sampler(sampler_binding, sampler)
        } else {
            bind_group
        };

        let mut occlusion_texture = None;
        let bind_group = if let Some(mtp) = self.occlusion {
            let texture_data = mtp
                .texture_data
                .ok_or(CoreError::EmptyNormalTexture(name.to_string()))?;
            occlusion_texture = Some(
                RenderTextureBuilder::new(&self.device)
                    .label(&format!("Texture: {name}"))
                    .bytes(&texture_data)
                    .format(mtp.format)
                    .usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST)
                    .build()?,
            );

            let view = occlusion_texture.as_ref().unwrap().view();
            let sampler = occlusion_texture.as_ref().unwrap().sampler()?;
            let view_binding = mtp.view_binding;
            let sampler_binding = mtp.sampler_binding;

            bind_group
                .entries_view(view_binding, view)
                .entries_sampler(sampler_binding, sampler)
        } else {
            bind_group
        };

        let mut emissive_texture = None;
        let bind_group = if let Some(mtp) = self.emissive {
            let texture_data = mtp
                .texture_data
                .ok_or(CoreError::EmptyNormalTexture(name.to_string()))?;
            emissive_texture = Some(
                RenderTextureBuilder::new(&self.device)
                    .label(&format!("Texture: {name}"))
                    .bytes(&texture_data)
                    .format(mtp.format)
                    .usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST)
                    .build()?,
            );

            let view = emissive_texture.as_ref().unwrap().view();
            let sampler = emissive_texture.as_ref().unwrap().sampler()?;
            let view_binding = mtp.view_binding;
            let sampler_binding = mtp.sampler_binding;

            bind_group
                .entries_view(view_binding, view)
                .entries_sampler(sampler_binding, sampler)
        } else {
            bind_group
        };

        let mut mr_texture = None;
        let bind_group = if let Some(mtp) = self.mr {
            let texture_data = mtp
                .texture_data
                .ok_or(CoreError::EmptyNormalTexture(name.to_string()))?;
            mr_texture = Some(
                RenderTextureBuilder::new(&self.device)
                    .label(&format!("Texture: {name}"))
                    .bytes(&texture_data)
                    .format(mtp.format)
                    .usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST)
                    .build()?,
            );

            let view = mr_texture.as_ref().unwrap().view();
            let sampler = mr_texture.as_ref().unwrap().sampler()?;
            let view_binding = mtp.view_binding;
            let sampler_binding = mtp.sampler_binding;

            bind_group
                .entries_view(view_binding, view)
                .entries_sampler(sampler_binding, sampler)
        } else {
            bind_group
        };

        let bind_group = bind_group.build()?;

        debug!(
            "
Build `{name}`:
    Normal texture: {normal_texture:#?},
    Diffuse texture: {diffuse_texture:#?},
    Emissive texture: {diffuse_texture:#?},
    MR texture: {diffuse_texture:#?},
    Occlusion texture: {diffuse_texture:#?},
    Bind group: {bind_group:#?},
            "
        );

        Ok(Material {
            id,
            name,
            diffuse_texture,
            normal_texture,
            occlusion_texture,
            mr_texture,
            emissive_texture,
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

    pub fn mr(mut self, mtp: MaterialTextureParams<'a>) -> Self {
        self.mr = Some(mtp);
        self
    }

    pub fn occlusion(mut self, mtp: MaterialTextureParams<'a>) -> Self {
        self.occlusion = Some(mtp);
        self
    }

    pub fn emissive(mut self, mtp: MaterialTextureParams<'a>) -> Self {
        self.emissive = Some(mtp);
        self
    }

    pub fn material_binding(mut self, binding: u32) -> Self {
        self.material_binding = binding;
        self
    }
}
