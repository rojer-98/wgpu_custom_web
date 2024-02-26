use derive_more::{Deref, DerefMut};
use image::{load_from_memory, GenericImageView};
use log::{debug, info};

use crate::{
    bind_group::{
        layout::{BindGroupLayout, BindGroupLayoutBuilder},
        BindGroup, BindGroupBuilder,
    },
    buffer::Buffer,
    errors::CoreError,
    traits::Builder,
};

#[derive(derivative::Derivative, Deref, DerefMut)]
#[derivative(Debug)]
pub struct RenderTexture {
    pub id: usize,

    pub view: wgpu::TextureView,
    pub sampler: Option<wgpu::Sampler>,

    bind_group: Option<BindGroup>,
    bind_group_layout: Option<BindGroupLayout>,

    dimensions: (u32, u32),
    #[derivative(Debug = "ignore")]
    data: Option<Vec<u8>>,
    size: wgpu::Extent3d,

    #[deref]
    #[deref_mut]
    texture: wgpu::Texture,
}

pub struct RenderTextureBuilder<'a> {
    id: Option<usize>,
    data: Option<&'a [u8]>,
    label: Option<&'a str>,
    is_normal_map: bool,
    is_sampler: bool,
    texture_size: Option<(u32, u32)>,
    depth_or_array_layers: u32,
    texture_desc: Option<wgpu::TextureDescriptor<'a>>,
    sampler_desc: Option<wgpu::SamplerDescriptor<'a>>,
    texture_view_desc: Option<wgpu::TextureViewDescriptor<'a>>,

    bind_group_binding: Option<u32>,
    view_layout_entry: Option<wgpu::BindGroupLayoutEntry>,
    sampler_layout_entry: Option<wgpu::BindGroupLayoutEntry>,

    device: &'a wgpu::Device,
}

impl<'a> Builder<'a> for RenderTextureBuilder<'a> {
    type Final = RenderTexture;

    fn new(device: &'a wgpu::Device) -> Self
    where
        Self: Sized,
    {
        Self {
            device,
            id: None,
            label: None,
            is_normal_map: false,
            is_sampler: true,
            data: None,
            texture_desc: None,
            sampler_desc: None,
            texture_view_desc: None,
            texture_size: None,
            depth_or_array_layers: 1,
            bind_group_binding: None,
            view_layout_entry: None,
            sampler_layout_entry: None,
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
            is_normal_map: false,
            is_sampler: true,
            data: None,
            texture_desc: None,
            sampler_desc: None,
            texture_view_desc: None,
            texture_size: None,
            depth_or_array_layers: 1,
            bind_group_binding: None,
            sampler_layout_entry: None,
            view_layout_entry: None,
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

        let bind_group_binding = self.bind_group_binding;
        let view_layout_entry = self
            .view_layout_entry
            .unwrap_or(wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            });
        let sampler_layout_entry =
            self.sampler_layout_entry
                .unwrap_or(wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                });

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

        let mut data = self.data.map(|d| d.to_vec());
        let (texture, dimensions, size) = if let Some(t_d) = texture_desc {
            let size = t_d.size;
            let dimensions = (size.width, size.height);

            (self.device.create_texture(&t_d), dimensions, size)
        } else {
            let dimensions = if let Some(d) = data.as_ref() {
                let img = load_from_memory(&*d)?;
                data = Some(img.to_rgba8().to_vec());

                img.dimensions()
            } else {
                texture_size.ok_or(CoreError::EmptyTextureSize(label.to_string()))?
            };

            let size = wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers,
            };

            let format = if self.is_normal_map {
                wgpu::TextureFormat::Rgba8Unorm
            } else {
                wgpu::TextureFormat::Rgba8UnormSrgb
            };

            let t_desc = texture_desc.unwrap_or(wgpu::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            (self.device.create_texture(&t_desc), dimensions, size)
        };
        let view = texture.create_view(&t_view_desc.unwrap_or_default());

        let sampler = if is_sampler {
            let s_desc = sampler_desc.unwrap_or(wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

            Some(self.device.create_sampler(&s_desc))
        } else {
            None
        };

        if let Some(bg_binding) = bind_group_binding {
            let bgl_name = format!("Bind group layout of `{texture_name}`");
            let mut bind_group_layout = BindGroupLayoutBuilder::new(self.device).label(&bgl_name);

            let bg_name = format!("Bind group of `{texture_name}`");
            let mut bind_group = BindGroupBuilder::new(self.device)
                .label(&bg_name)
                .binding(bg_binding);

            bind_group_layout = bind_group_layout.entries(view_layout_entry);
            bind_group = bind_group.entries_view(view_layout_entry.binding, &view);

            if let Some(sampler) = sampler.as_ref() {
                bind_group_layout = bind_group_layout.entries(sampler_layout_entry);
                bind_group = bind_group.entries_sampler(sampler_layout_entry.binding, sampler);
            }

            let bind_group_layout = bind_group_layout.build()?;
            let bind_group = bind_group.layout(&bind_group_layout).build()?;

            Ok(RenderTexture {
                id,
                view,
                sampler,
                dimensions,
                data,
                size,
                texture,
                bind_group: Some(bind_group),
                bind_group_layout: Some(bind_group_layout),
            })
        } else {
            Ok(RenderTexture {
                id,
                texture,
                view,
                sampler,
                dimensions,
                size,
                data,
                bind_group_layout: None,
                bind_group: None,
            })
        }
    }
}

impl<'a> RenderTextureBuilder<'a> {
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn bytes(mut self, data: &'a [u8]) -> Self {
        self.data = Some(data);
        self
    }

    pub fn is_normal_map(mut self, is_normal_map: bool) -> Self {
        self.is_normal_map = is_normal_map;
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

    pub fn bind_group_binding(mut self, bind_group_binding: u32) -> Self {
        self.bind_group_binding = Some(bind_group_binding);
        self
    }

    pub fn view_layout_entry(mut self, view_layout_entry: wgpu::BindGroupLayoutEntry) -> Self {
        self.view_layout_entry = Some(view_layout_entry);
        self
    }

    pub fn sampler_layout_entry(
        mut self,
        sampler_layout_entry: wgpu::BindGroupLayoutEntry,
    ) -> Self {
        self.sampler_layout_entry = Some(sampler_layout_entry);
        self
    }
}

impl RenderTexture {
    pub fn bind_group(&self) -> Result<&BindGroup, CoreError> {
        self.bind_group
            .as_ref()
            .ok_or(CoreError::EmptyBindGroup(format!(
                "Render texture: {}",
                self.id
            )))
    }

    pub fn bind_group_layout(&self) -> Result<&BindGroupLayout, CoreError> {
        self.bind_group_layout
            .as_ref()
            .ok_or(CoreError::EmptyBindGroupLayout(format!(
                "Render texture: {}",
                self.id
            )))
    }

    pub fn store_to_memory(&self, queue: &wgpu::Queue) {
        if let Some(img_data) = self.data.as_ref() {
            let aspect = wgpu::TextureAspect::All;
            let components = self.format().components_with_aspect(aspect) as u32;

            info!(
                "Store to memory: aspect {aspect:?}, components {components:?}, size: {:?}",
                self.size
            );

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    aspect,
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                img_data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(components * self.dimensions.0),
                    rows_per_image: Some(self.dimensions.1),
                },
                self.size,
            );
        }
    }

    pub fn load_to_buffer(
        &self,
        queue: &wgpu::Queue,
        mut encoder: wgpu::CommandEncoder,
        output_buffer: &Buffer,
    ) {
        let aspect = wgpu::TextureAspect::All;
        let components = self.format().components_with_aspect(aspect) as u32;

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect,
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(components * self.dimensions.0),
                    rows_per_image: Some(self.dimensions.1),
                },
            },
            self.size,
        );

        queue.submit(Some(encoder.finish()));
    }
}
