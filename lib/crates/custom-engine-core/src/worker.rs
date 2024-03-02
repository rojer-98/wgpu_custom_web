mod context_impls;
mod inner;

use crate::{
    buffer::Buffer,
    context::Context,
    errors::CoreError,
    runtime::RuntimeKind,
    texture::{RenderTexture, TextureKind},
};

#[derive(Debug)]
pub enum View {
    Surface(wgpu::SurfaceTexture),
    Texture(RenderTexture, Buffer),
}

impl View {
    fn texture_view(&self) -> wgpu::TextureView {
        match self {
            View::Surface(s) => s.texture.create_view(&Default::default()),
            View::Texture(t, _) => t.create_view(&Default::default()),
        }
    }
}

#[derive(Debug)]
pub struct Worker<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) queue: &'a wgpu::Queue,
    pub(crate) limits: wgpu::Limits,

    pub(crate) runtime_kind: RuntimeKind<'a>,
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
        runtime_kind: RuntimeKind<'a>,
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        limits: wgpu::Limits,
        view: Option<View>,
        context: Context,
    ) -> Result<Self, CoreError> {
        let format = match runtime_kind {
            RuntimeKind::Winit(_) => TextureKind::Surface.into(),
            RuntimeKind::Texture(_, _) => TextureKind::Render.into(),
        };

        Ok(Self {
            size,
            scale_factor,
            runtime_kind,
            format,
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
