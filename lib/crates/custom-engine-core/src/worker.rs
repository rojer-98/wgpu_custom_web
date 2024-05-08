mod context_impls;
mod inner;

use crate::{
    buffer::Buffer,
    context::Context,
    errors::CoreError,
    runtime::{ImageFormat, SurfaceProperties},
    texture::{RenderTexture, TextureKind},
};

#[derive(Debug)]
pub(crate) struct ViewTexture {
    render_texture: RenderTexture,
    buffer: Buffer,
    image_format: ImageFormat,
    path_to_save: String,
}

#[derive(Debug)]
pub(crate) enum View {
    Surface(wgpu::SurfaceTexture),
    Texture(ViewTexture),
}

impl View {
    fn texture_view(&self) -> wgpu::TextureView {
        match self {
            View::Surface(s) => s.texture.create_view(&Default::default()),
            View::Texture(ViewTexture { render_texture, .. }) => {
                render_texture.create_view(&Default::default())
            }
        }
    }
}

#[derive(Debug)]
pub struct Worker<'a> {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) limits: wgpu::Limits,

    pub(crate) surface_properties: SurfaceProperties<'a>,
    pub(crate) context: Context,

    format: wgpu::TextureFormat,
    size: (u32, u32),
    scale_factor: f64,

    view: Option<View>,
}

impl<'a> Worker<'a> {
    pub(crate) fn new(
        size: (u32, u32),
        scale_factor: f64,
        surface_properties: SurfaceProperties<'a>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        limits: wgpu::Limits,
        view: Option<View>,
        context: Context,
    ) -> Result<Self, CoreError> {
        Ok(Self {
            size,
            scale_factor,
            surface_properties,
            format: TextureKind::Surface.into(),
            device,
            queue,
            limits,
            view,
            context,
        })
    }

    pub fn into_context(self) -> Context {
        self.context
    }
}
