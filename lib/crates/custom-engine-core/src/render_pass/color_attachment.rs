use derive_more::{Deref, DerefMut};
use log::debug;

use crate::errors::CoreError;

#[derive(Debug, Deref, DerefMut)]
pub struct ColorAttachments<'a>(Vec<ColorAttachment<'a>>);

impl<'a> ColorAttachments<'a> {
    pub fn into_render_pass(self) -> Vec<Option<wgpu::RenderPassColorAttachment<'a>>> {
        self.0
            .into_iter()
            .map(ColorAttachment::into_render_pass)
            .collect()
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct ColorAttachment<'a> {
    pub id: usize,

    #[deref]
    #[deref_mut]
    inner_ca: wgpu::RenderPassColorAttachment<'a>,
}

impl<'a> ColorAttachment<'a> {
    pub fn into_render_pass(self) -> Option<wgpu::RenderPassColorAttachment<'a>> {
        Some(self.inner_ca)
    }
}

#[derive(Debug)]
pub struct ColorAttachmentBuilder<'a> {
    id: Option<usize>,
    label: Option<&'a str>,
    view: Option<&'a wgpu::TextureView>,
    ops: Option<wgpu::Operations<wgpu::Color>>,
}

impl<'a> ColorAttachmentBuilder<'a> {
    pub fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            view: None,
            id: None,
            label: None,
            ops: None,
        }
    }

    pub fn new_indexed(id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            view: None,
            id: Some(id),
            label: None,
            ops: None,
        }
    }

    pub fn build(self) -> Result<ColorAttachment<'a>, CoreError>
    where
        Self: Sized,
    {
        let id = self.id.unwrap_or_default();
        let color_attach_name = format!("Color attachment: {id}");

        let label = self.label.unwrap_or(&color_attach_name);
        let view = self
            .view
            .ok_or(CoreError::EmptyTextureView(label.to_string()))?;
        let ops = self.ops.unwrap_or(wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
        });

        debug!(
            "
Build `{label}`:
    View: {view:#?},
    Ops: {ops:#?},"
        );

        let inner_ca = wgpu::RenderPassColorAttachment {
            view,
            ops,
            resolve_target: None,
        };

        Ok(ColorAttachment { id, inner_ca })
    }
}

impl<'a> ColorAttachmentBuilder<'a> {
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn view(mut self, view: &'a wgpu::TextureView) -> Self {
        self.view = Some(view);
        self
    }

    pub fn ops(mut self, ops: wgpu::Operations<wgpu::Color>) -> Self {
        self.ops = Some(ops);
        self
    }
}
