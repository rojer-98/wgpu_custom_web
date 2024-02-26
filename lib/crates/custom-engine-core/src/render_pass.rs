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

#[derive(Debug, Clone)]
pub struct RenderStage<'a> {
    pipeline: &'a Pipeline,
    vertex_buffer: Option<&'a Buffer>,
    index_buffer: Option<&'a Buffer>,
    bind_groups: Option<Vec<&'a BindGroup>>,
    model: Option<&'a Model>,

    instances: Range<u32>,
    vertices: Range<u32>,
}

impl<'a> RenderStage<'a> {
    pub fn new(pipeline: &'a Pipeline, instances: Range<u32>, vertices: Range<u32>) -> Self {
        Self {
            pipeline,
            vertices,
            instances,
            model: None,
            bind_groups: None,
            index_buffer: None,
            vertex_buffer: None,
        }
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
    color_attachments: Option<ColorAttachmentBuilder<'a>>,
    depth_stencil: Option<DepthStencilAttachmentBuilder<'a>>,
    query_set: Option<QuerySet>,
    render_stages: BTreeMap<usize, RenderStage<'a>>,

    viewport: Option<ViewportRect>,
    scissors: Option<ScissorsRect>,
    blend_constant: Option<wgpu::Color>,
    stencil_reference: Option<u32>,

    copy_params: Option<CopyTextureParams<'a>>,

    device: &'a wgpu::Device,
}

impl<'a> RenderPass<'a> {
    pub fn new(device: &'a wgpu::Device, id: usize) -> Self {
        Self {
            id,
            device,
            depth_stencil: None,
            label: None,
            color_attachments: None,
            viewport: None,
            scissors: None,
            blend_constant: None,
            stencil_reference: None,
            copy_params: None,
            query_set: None,
            render_stages: BTreeMap::default(),
        }
    }

    pub fn query_set(mut self, query_set: QuerySet) -> Self {
        self.query_set = Some(query_set);
        self
    }

    pub fn copy_params(mut self, copy_params: CopyTextureParams<'a>) -> Self {
        self.copy_params = Some(copy_params);
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

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
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

    pub fn render_stage(mut self, index: usize, render_stage: RenderStage<'a>) -> Self {
        if let Some(old_stage) = self.render_stages.insert(index, render_stage) {
            warn!("Old render stage was presented in `render_pass`: {old_stage:#?}");
        }
        self
    }

    pub fn render_stages(mut self, render_stages: Vec<RenderStage<'a>>) -> Self {
        if let Some(last) = self.render_stages.keys().last() {
            let mut index = *last;

            render_stages.into_iter().for_each(|render_stage| {
                if let Some(old_stage) = self.render_stages.insert(index, render_stage) {
                    warn!("Old render stage was presented in `render_pass`: {old_stage:#?}");
                }

                index += 1;
            });
        } else {
            self.render_stages.append(
                &mut render_stages
                    .into_iter()
                    .enumerate()
                    .collect::<BTreeMap<_, _>>(),
            )
        }

        self
    }

    pub fn render(self, queue: &'a wgpu::Queue) -> Result<(), CoreError> {
        let id = self.id;
        let render_pass_name = format!("Render pass: {id}");

        let label = self.label.unwrap_or(&render_pass_name);
        let viewport = self.viewport;
        let scissors = self.scissors;
        let stencil_reference = self.stencil_reference;
        let blend_constant = self.blend_constant;
        let copy_params = self.copy_params;

        let color_attachments = self
            .color_attachments
            .ok_or(CoreError::EmptyRenderPassColorAttachemnts(
                label.to_string(),
            ))?
            .build()?
            .into_render_pass();
        let depth_stencil_attachment = self
            .depth_stencil
            .and_then(|d_s_b| d_s_b.build().ok())
            .and_then(|d_s| d_s.into_render_pass());
        let occlusion_query_set = self.query_set.as_deref();

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

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&format!("Command Encoder of `{label}`")),
            });

        debug!(
            "
Process `{label}`:
    Viewport: {viewport:#?},
    Scissors: {scissors:#?},
    Stencil Reference: {stencil_reference:#?},
    Blend Constant: {blend_constant:#?},
    Copy Params: {copy_params:#?}"
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(label),
                color_attachments: &[color_attachments],
                timestamp_writes: None,
                occlusion_query_set,
                depth_stencil_attachment,
            });

            for (i, r_s) in self.render_stages {
                let RenderStage {
                    pipeline,
                    vertex_buffer,
                    index_buffer,
                    bind_groups,
                    vertices,
                    instances,
                    model,
                } = r_s;
                let indexed = r_s.index_buffer.is_some();

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
                render_pass.set_pipeline(pipeline);
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
Process `render stage: {i}`
    Pipeline: {pipeline:#?},
    Model: {model:#?},
    Vertex Buffer: {vertex_buffer:#?},
    Index Buffer: {index_buffer:#?},
    Bind Groups: {bind_groups:#?},
    Vertices: {vertices:?},
    Instances: {instances:?},
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
                    render_pass.draw_indexed(vertices, 0, instances);
                } else {
                    render_pass.draw(vertices, instances);
                }
            }
        }

        if let Some(c_p) = copy_params {
            let CopyTextureParams { buffer, texture } = c_p;

            texture.load_to_buffer(queue, encoder, buffer);
        } else {
            queue.submit(once(encoder.finish()));
        }

        Ok(())
    }
}
