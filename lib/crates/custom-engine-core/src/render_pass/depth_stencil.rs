use derive_more::{Deref, DerefMut};
use log::debug;

use crate::errors::CoreError;

#[derive(Debug, Deref, DerefMut)]
pub struct DepthStencilAttachment<'a> {
    pub id: usize,

    #[deref]
    #[deref_mut]
    inner_ds: wgpu::RenderPassDepthStencilAttachment<'a>,
}

impl<'a> DepthStencilAttachment<'a> {
    pub fn into_render_pass(self) -> Option<wgpu::RenderPassDepthStencilAttachment<'a>> {
        Some(self.inner_ds)
    }
}

#[derive(Debug, Default)]
pub struct DepthStencilAttachmentBuilder<'a> {
    id: Option<usize>,
    label: Option<&'a str>,
    view: Option<&'a wgpu::TextureView>,
    depth_ops: Option<wgpu::Operations<f32>>,
    stencil_ops: Option<wgpu::Operations<u32>>,
}

impl<'a> DepthStencilAttachmentBuilder<'a> {
    pub fn new() -> Self
    where
        Self: Sized,
    {
        Default::default()
    }

    pub fn new_indexed(id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            id: Some(id),
            ..Default::default()
        }
    }

    pub fn build(self) -> Result<DepthStencilAttachment<'a>, CoreError>
    where
        Self: Sized,
    {
        let id = self.id.unwrap_or_default();
        let depth_stencil_name = format!("Depth stencil attachment: {id}");

        let label = self.label.unwrap_or(&depth_stencil_name);
        let view = self
            .view
            .ok_or(CoreError::EmptyTextureView(label.to_string()))?;
        let depth_ops = self.depth_ops.unwrap_or(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.),
            store: wgpu::StoreOp::Store,
        });
        let stencil_ops = self.stencil_ops;

        debug!(
            "
Build `{label}`:
    View: {view:#?},
    Depth_ops: {depth_ops:#?},"
        );

        let inner_ds = wgpu::RenderPassDepthStencilAttachment {
            view,
            depth_ops: Some(depth_ops),
            stencil_ops,
        };

        Ok(DepthStencilAttachment { id, inner_ds })
    }
}

impl<'a> DepthStencilAttachmentBuilder<'a> {
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn view(mut self, view: &'a wgpu::TextureView) -> Self {
        self.view = Some(view);
        self
    }

    pub fn depth_ops(mut self, ops: wgpu::Operations<f32>) -> Self {
        self.depth_ops = Some(ops);
        self
    }

    pub fn stencil_ops(mut self, ops: wgpu::Operations<u32>) -> Self {
        self.stencil_ops = Some(ops);
        self
    }
}
