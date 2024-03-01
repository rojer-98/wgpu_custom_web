pub mod color_attachment;
pub mod depth_stencil;
pub mod query_set;

use std::{collections::BTreeMap, iter::once, ops::Range};

use log::{debug, warn};

use crate::{
    bind_group::BindGroup,
    buffer::Buffer,
    errors::CoreError,
    model::Model,
    pipeline::Pipeline,
    render_pass::{
        color_attachment::ColorAttachmentBuilder, depth_stencil::DepthStencilAttachmentBuilder,
        query_set::QuerySet,
    },
    texture::CopyTextureParams,
};

#[derive(Debug)]
enum Stage<'a> {
    Render(RenderStage<'a>),
    Compute(ComputeStage<'a>),
}

impl<'a> Stage<'a> {
    pub fn process(
        self,
        index: usize,
        label: &str,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), CoreError> {
        use Stage::*;

        match self {
            Render(r_s) => {
                let RenderStage {
                    pipeline,
                    vertex_buffer,
                    index_buffer,
                    bind_groups,
                    instances,
                    model,
                    entities,
                    base_vertex,
                    color_attachments,
                    depth_stencil,
                    query_set,
                    viewport,
                    scissors,
                    blend_constant,
                    stencil_reference,
                } = r_s;

                let color_attachments = color_attachments
                    .ok_or(CoreError::EmptyRenderPassColorAttachemnts(
                        label.to_string(),
                    ))?
                    .build()?
                    .into_render_pass();
                let depth_stencil_attachment = depth_stencil
                    .and_then(|d_s_b| d_s_b.build().ok())
                    .and_then(|d_s| d_s.into_render_pass());
                let occlusion_query_set = query_set.as_deref();

                /*
                    let query_set_desc_label = format!("QuerySet Descriptior Label: {label}");
                    let query_set_desc = wgpu::QuerySetDescriptor {
                        label: Some(&query_set_desc_label),
                        ty: wgpu::QueryType::Timestamp,
                        count: wgpu::QUERY_SET_MAX_QUERIES - 1,
                    };

                    let timestamp_query_set = self.device.create_query_set(&query_set_desc);
                    let timestamp_writes = Some(wgpu::RenderPassTimestampWrites {
                        query_set: &timestamp_query_set,
                        beginning_of_pass_write_index: Some(0),
                        end_of_pass_write_index: None,
                    });

                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some(label),
                        color_attachments: &[color_attachments],
                        timestamp_writes,
                        occlusion_query_set,
                        depth_stencil_attachment,
                    })
                */

                let entities = entities.ok_or(CoreError::EmptyEntities(index))?;
                let instances = instances.ok_or(CoreError::EmptyInstances(index))?;
                let indexed = r_s.index_buffer.is_some();

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(label),
                    color_attachments: &[color_attachments],
                    timestamp_writes: None,
                    occlusion_query_set,
                    depth_stencil_attachment,
                });

                if let Some(vb) = vertex_buffer.as_ref() {
                    render_pass.set_vertex_buffer(vb.binding, vb.slice(..));
                }
                if let Some(ib) = index_buffer.as_ref() {
                    render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint16);
                }

                if let Some(b_gs) = bind_groups.as_ref() {
                    b_gs.iter()
                        .for_each(|bg| render_pass.set_bind_group(bg.binding, bg, &[]));
                }
                render_pass.set_pipeline(
                    pipeline
                        .render()
                        .ok_or(CoreError::NotRenderPipeline(label.to_string()))?,
                );
                if let Some(v) = viewport.as_ref() {
                    let (x, y) = v.coords;
                    let (w, h) = v.size;

                    render_pass.set_viewport(x, y, w, h, v.min_depth, v.max_depth);
                }
                if let Some(s) = scissors.as_ref() {
                    let (x, y) = s.coords;
                    let (w, h) = s.size;

                    render_pass.set_scissor_rect(x, y, w, h);
                }
                if let Some(b_c) = blend_constant.as_ref() {
                    render_pass.set_blend_constant(*b_c);
                }
                if let Some(s_r) = stencil_reference.as_ref() {
                    render_pass.set_stencil_reference(*s_r);
                }

                debug!(
                    "
Process `render stage: {index}`
    Pipeline: {pipeline:#?},
    Model: {model:#?},
    Vertex Buffer: {vertex_buffer:#?},
    Index Buffer: {index_buffer:#?},
    Bind Groups: {bind_groups:#?},
    Entities: {entities:?},
    Instances: {instances:?},
    Viewport: {viewport:#?},
    Scissors: {scissors:#?},
    Stencil Reference: {stencil_reference:#?},
    Blend Constant: {blend_constant:#?},
"
                );

                if let Some(m) = model {
                    let meshes = m.meshes();
                    let materials = m.materials();

                    for mesh in meshes {
                        let material = &materials[mesh.material];
                        let bg = material.bind_group();

                        let v_b = mesh.vertex_buffer();
                        let i_b = mesh.index_buffer();

                        render_pass.set_vertex_buffer(v_b.binding, v_b.slice(..));
                        render_pass.set_index_buffer(i_b.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.set_bind_group(bg.binding, &bg, &[]);

                        render_pass.draw_indexed(0..mesh.num_elements, 0, instances.clone());
                    }
                } else if indexed {
                    let base_vertex = base_vertex.unwrap_or(0);
                    render_pass.draw_indexed(entities, base_vertex, instances);
                } else {
                    render_pass.draw(entities, instances);
                }
            }
            Compute(c_s) => {
                let ComputeStage {
                    pipeline,
                    bind_groups,
                    x_dimension,
                    y_dimension,
                    z_dimension,
                } = c_s;

                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(label),
                    timestamp_writes: None,
                });

                compute_pass.set_pipeline(
                    pipeline
                        .compute()
                        .ok_or(CoreError::NotComputePipeline(label.to_string()))?,
                );
                if let Some(b_gs) = bind_groups.as_ref() {
                    b_gs.iter()
                        .for_each(|bg| compute_pass.set_bind_group(bg.binding, bg, &[]));
                }
                compute_pass.dispatch_workgroups(x_dimension, y_dimension, z_dimension);
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ComputeStage<'a> {
    pipeline: &'a Pipeline,

    bind_groups: Option<Vec<&'a BindGroup>>,

    x_dimension: u32,
    y_dimension: u32,
    z_dimension: u32,
}

impl<'a> ComputeStage<'a> {
    pub fn new(pipeline: &'a Pipeline) -> Self {
        Self {
            pipeline,
            bind_groups: None,
            x_dimension: 1,
            y_dimension: 1,
            z_dimension: 1,
        }
    }

    pub fn bind_groups(mut self, bind_groups: Vec<&'a BindGroup>) -> Self {
        self.bind_groups = Some(bind_groups);
        self
    }

    pub fn x_dimension(mut self, x_dimension: u32) -> Self {
        self.x_dimension = x_dimension;
        self
    }

    pub fn y_dimension(mut self, y_dimension: u32) -> Self {
        self.y_dimension = y_dimension;
        self
    }

    pub fn z_dimension(mut self, z_dimension: u32) -> Self {
        self.z_dimension = z_dimension;
        self
    }
}

#[derive(Debug)]
pub struct RenderStage<'a> {
    pipeline: &'a Pipeline,

    vertex_buffer: Option<&'a Buffer>,
    index_buffer: Option<&'a Buffer>,
    bind_groups: Option<Vec<&'a BindGroup>>,
    model: Option<&'a Model>,

    instances: Option<Range<u32>>,
    base_vertex: Option<i32>,
    entities: Option<Range<u32>>,

    color_attachments: Option<ColorAttachmentBuilder<'a>>,
    depth_stencil: Option<DepthStencilAttachmentBuilder<'a>>,
    query_set: Option<QuerySet>,

    viewport: Option<ViewportRect>,
    scissors: Option<ScissorsRect>,
    blend_constant: Option<wgpu::Color>,
    stencil_reference: Option<u32>,
}

impl<'a> RenderStage<'a> {
    pub fn new(pipeline: &'a Pipeline) -> Self {
        Self {
            pipeline,

            model: None,
            bind_groups: None,
            index_buffer: None,
            vertex_buffer: None,

            instances: None,
            base_vertex: None,
            entities: None,

            query_set: None,
            depth_stencil: None,
            color_attachments: None,

            viewport: None,
            scissors: None,
            blend_constant: None,
            stencil_reference: None,
        }
    }

    pub fn query_set(mut self, query_set: QuerySet) -> Self {
        self.query_set = Some(query_set);
        self
    }

    pub fn color_attachments_builder(
        mut self,
        color_attachments: ColorAttachmentBuilder<'a>,
    ) -> Self {
        self.color_attachments = Some(color_attachments);
        self
    }

    pub fn depth_stencil_builder(
        mut self,
        depth_stencil: DepthStencilAttachmentBuilder<'a>,
    ) -> Self {
        self.depth_stencil = Some(depth_stencil);
        self
    }

    pub fn instances(mut self, instances: Range<u32>) -> Self {
        self.instances = Some(instances);
        self
    }

    pub fn entities(mut self, entities: Range<u32>) -> Self {
        self.entities = Some(entities);
        self
    }

    pub fn base_vertex(mut self, base_vertex: i32) -> Self {
        self.base_vertex = Some(base_vertex);
        self
    }

    pub fn model(mut self, model: &'a Model) -> Self {
        self.model = Some(model);
        self
    }

    pub fn vertex_buffer(mut self, vertex_buffer: &'a Buffer) -> Self {
        self.vertex_buffer = Some(vertex_buffer);
        self
    }

    pub fn index_buffer(mut self, index_buffer: &'a Buffer) -> Self {
        self.index_buffer = Some(index_buffer);
        self
    }

    pub fn bind_groups(mut self, bind_groups: Vec<&'a BindGroup>) -> Self {
        self.bind_groups = Some(bind_groups);
        self
    }

    pub fn viewport(mut self, viewport: ViewportRect) -> Self {
        self.viewport = Some(viewport);
        self
    }

    pub fn scissors(mut self, scissors: ScissorsRect) -> Self {
        self.scissors = Some(scissors);
        self
    }

    pub fn blend_constant(mut self, color: wgpu::Color) -> Self {
        self.blend_constant = Some(color);
        self
    }

    pub fn stencil_reference(mut self, index: u32) -> Self {
        self.stencil_reference = Some(index);
        self
    }
}

#[derive(Debug, Clone)]
pub struct ScissorsRect {
    pub coords: (u32, u32),
    pub size: (u32, u32),
}

#[derive(Debug, Clone)]
pub struct ViewportRect {
    pub coords: (f32, f32),
    pub size: (f32, f32),
    pub min_depth: f32,
    pub max_depth: f32,
}

#[derive(Debug)]
pub struct RenderPass<'a> {
    pub id: usize,

    label: Option<&'a str>,
    stages: BTreeMap<usize, Stage<'a>>,

    copy_params: Option<CopyTextureParams<'a>>,

    device: &'a wgpu::Device,
}

impl<'a> RenderPass<'a> {
    pub fn new(device: &'a wgpu::Device, id: usize) -> Self {
        Self {
            id,
            device,
            label: None,
            copy_params: None,

            stages: BTreeMap::default(),
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn copy_params(mut self, copy_params: CopyTextureParams<'a>) -> Self {
        self.copy_params = Some(copy_params);
        self
    }

    pub fn render_stage(self, index: usize, stage: RenderStage<'a>) -> Self {
        self.stage(index, Stage::Render(stage))
    }

    pub fn compute_stage(self, index: usize, stage: ComputeStage<'a>) -> Self {
        self.stage(index, Stage::Compute(stage))
    }

    pub fn render(self, queue: &'a wgpu::Queue) -> Result<(), CoreError> {
        let id = self.id;
        let render_pass_name = format!("Render pass: {id}");

        let label = self.label.unwrap_or(&render_pass_name);
        let copy_params = self.copy_params;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&format!("Command Encoder of `{label}`")),
            });

        debug!(
            "
Process `{label}`:
    Copy Params: {copy_params:#?}
    "
        );

        for (i, s) in self.stages {
            s.process(i, label, &mut encoder)?;
        }

        if let Some(c_p) = copy_params {
            c_p.process(&mut encoder);
        }

        queue.submit(once(encoder.finish()));

        Ok(())
    }

    // Helpers
    fn stage(mut self, index: usize, stage: Stage<'a>) -> Self {
        if let Some(old_stage) = self.stages.insert(index, stage) {
            warn!("Old render stage was presented in `render_pass`: {old_stage:#?}");
        }
        self
    }
}
