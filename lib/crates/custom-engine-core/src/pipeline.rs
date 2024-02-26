pub mod layout;

use std::num::NonZeroU32;

use derive_more::{Deref, DerefMut};
use log::debug;

use crate::{errors::CoreError, pipeline::layout::PipelineLayout, shader::Shader, traits::Builder};

#[derive(Debug, Deref, DerefMut)]
pub struct Pipeline {
    pub id: usize,
    pub label: String,
    pub primitive: wgpu::PrimitiveState,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    pub multisample: wgpu::MultisampleState,
    pub multiview: Option<NonZeroU32>,

    #[deref]
    #[deref_mut]
    inner_pipeline: wgpu::RenderPipeline,
}

pub struct PipelineBuilder<'a> {
    id: Option<usize>,
    label: Option<&'a str>,
    layout: Option<&'a PipelineLayout>,
    shader: Option<&'a Shader>,
    primitive: Option<&'a wgpu::PrimitiveState>,
    depth_stencil: Option<&'a wgpu::DepthStencilState>,
    multisample: Option<&'a wgpu::MultisampleState>,
    multiview: Option<u32>,

    device: &'a wgpu::Device,
}

impl<'a> Builder<'a> for PipelineBuilder<'a> {
    type Final = Pipeline;

    fn new(device: &'a wgpu::Device) -> Self
    where
        Self: Sized,
    {
        Self {
            device,
            multiview: None,
            multisample: None,
            depth_stencil: None,
            primitive: None,
            shader: None,
            layout: None,
            label: None,
            id: None,
        }
    }

    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            device,
            multiview: None,
            multisample: None,
            depth_stencil: None,
            primitive: None,
            shader: None,
            layout: None,
            label: None,
            id: Some(id),
        }
    }

    fn build(self) -> Result<Self::Final, CoreError>
    where
        Self: Sized,
    {
        let id = self.id.unwrap_or_default();
        let pipeline_name = format!("Pipeline: {id}");

        let label = self.label.unwrap_or(&pipeline_name);
        let multiview = self.multiview.and_then(NonZeroU32::new);
        let multisample = *self
            .multisample
            .ok_or(CoreError::EmptyPipelineMultisample(label.to_string()))?;
        let depth_stencil = self.depth_stencil.cloned();
        let primitive = *self
            .primitive
            .ok_or(CoreError::EmptyPipelinePrimitive(label.to_string()))?;
        let layout = self
            .layout
            .ok_or(CoreError::EmptyLayout(label.to_string()))?;
        let shader = self
            .shader
            .ok_or(CoreError::EmptyPipelineVertex(label.to_string()))?;

        debug!(
            "
Build `{label}`: 
    Multiview: {multiview:#?},
    Multisample: {multisample:#?},
    Depth stencil: {depth_stencil:#?},
    Primitive: {primitive:#?},
    Layout: {layout:#?},
    Shader: {shader:#?}"
        );

        let inner_pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(layout),
                multisample,
                depth_stencil: depth_stencil.clone(),
                primitive,
                vertex: shader.make_vertex_state(),
                fragment: Some(shader.make_fragment_state()),
                multiview,
            });

        Ok(Pipeline {
            id,
            label: label.to_string(),
            primitive,
            depth_stencil,
            multisample,
            multiview,

            inner_pipeline,
        })
    }
}

impl<'a> PipelineBuilder<'a> {
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn multiview(mut self, multiview: u32) -> Self {
        self.multiview = Some(multiview);
        self
    }

    pub fn multisample(mut self, multisample: &'a wgpu::MultisampleState) -> Self {
        self.multisample = Some(multisample);
        self
    }

    pub fn depth_stencil(mut self, depth_stencil: &'a wgpu::DepthStencilState) -> Self {
        self.depth_stencil = Some(depth_stencil);
        self
    }

    pub fn primitive(mut self, primitive: &'a wgpu::PrimitiveState) -> Self {
        self.primitive = Some(primitive);
        self
    }

    pub fn layout(mut self, layout: &'a PipelineLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    pub fn shader(mut self, shader: &'a Shader) -> Self {
        self.shader = Some(shader);
        self
    }
}
