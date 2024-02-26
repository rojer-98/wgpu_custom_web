use derive_more::{Deref, DerefMut};
use log::debug;

use crate::{errors::CoreError, traits::Builder};

#[derive(Debug, Deref, DerefMut)]
pub struct DepthTexture {
    pub id: usize,
    pub view: wgpu::TextureView,
    pub sampler: Option<wgpu::Sampler>,

    #[deref]
    #[deref_mut]
    texture: wgpu::Texture,
}

pub struct DepthTextureBuilder<'a> {
    id: Option<usize>,
    data: Option<&'a [u8]>,
    label: Option<&'a str>,
    is_sampler: bool,
    texture_size: Option<(u32, u32)>,
    depth_or_array_layers: u32,
    texture_desc: Option<wgpu::TextureDescriptor<'a>>,
    sampler_desc: Option<wgpu::SamplerDescriptor<'a>>,
    texture_view_desc: Option<wgpu::TextureViewDescriptor<'a>>,

    device: &'a wgpu::Device,
}

impl<'a> Builder<'a> for DepthTextureBuilder<'a> {
    type Final = DepthTexture;

    fn new(device: &'a wgpu::Device) -> Self
    where
        Self: Sized,
    {
        Self {
            device,
            id: None,
            label: None,
            is_sampler: true,
            data: None,
            texture_desc: None,
            sampler_desc: None,
            texture_view_desc: None,
            texture_size: None,
            depth_or_array_layers: 1,
        }
    }

    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            device,
            id: Some(id),
            label: None,
            is_sampler: true,
            data: None,
            texture_desc: None,
            sampler_desc: None,
            texture_view_desc: None,
            texture_size: None,
            depth_or_array_layers: 1,
        }
    }

    fn build(self) -> Result<Self::Final, CoreError>
    where
        Self: Sized,
    {
        let id = self.id.unwrap_or_default();
        let texture_name = format!("Texture: {id}");

        let label = self.label.unwrap_or(&texture_name);
        let depth_or_array_layers = self.depth_or_array_layers;

        let texture_desc = self.texture_desc;
        let is_sampler = self.is_sampler;
        let sampler_desc = self.sampler_desc;
        let t_view_desc = self.texture_view_desc;
        let texture_size = self.texture_size;

        debug!(
            "
Build `{label}`: 
    Is Sampler: {is_sampler},
    Depth layers: {depth_or_array_layers},
    Texture description: {texture_desc:#?},
    Texture view description: {t_view_desc:#?},
    Sampler description: {sampler_desc:#?},
    "
        );

        let dimensions = texture_size.ok_or(CoreError::EmptyTextureSize(label.to_string()))?;
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers,
        };
        let t_desc = texture_desc.unwrap_or(wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture = self.device.create_texture(&t_desc);
        let view = texture.create_view(&t_view_desc.unwrap_or_default());

        let sampler = if is_sampler {
            let s_desc = sampler_desc.unwrap_or(wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual),
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            });

            Some(self.device.create_sampler(&s_desc))
        } else {
            None
        };

        Ok(DepthTexture {
            id,
            texture,
            view,
            sampler,
        })
    }
}

impl<'a> DepthTextureBuilder<'a> {
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn bytes(mut self, data: &'a [u8]) -> Self {
        self.data = Some(data);
        self
    }

    pub fn is_sampler(mut self, is_sampler: bool) -> Self {
        self.is_sampler = is_sampler;
        self
    }

    pub fn texture_size(mut self, texture_size: (u32, u32)) -> Self {
        self.texture_size = Some(texture_size);
        self
    }

    pub fn sampler_desc(mut self, sampler_desc: wgpu::SamplerDescriptor<'a>) -> Self {
        self.sampler_desc = Some(sampler_desc);
        self
    }

    pub fn texture_desc(mut self, texture_desc: wgpu::TextureDescriptor<'a>) -> Self {
        self.texture_desc = Some(texture_desc);
        self
    }

    pub fn texture_view_desc(mut self, texture_view_desc: wgpu::TextureViewDescriptor<'a>) -> Self {
        self.texture_view_desc = Some(texture_view_desc);
        self
    }

    pub fn depth_or_array_layers(mut self, depth_or_array_layers: u32) -> Self {
        self.depth_or_array_layers = depth_or_array_layers;
        self
    }
}
