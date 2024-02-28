use custom_engine_core::{
    errors::CoreError,
    render_pass::color_attachment::ColorAttachmentBuilder,
    render_pass::RenderStage,
    traits::{Builder, RenderWorker},
    worker::Worker,
};

use anyhow::Result;
use winit::event::WindowEvent;

use crate::files::{ShaderFiles, ShaderKind};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];

pub struct SimpleRenderToTexture {
    vb_id: usize,
    p_id: usize,
}

impl RenderWorker for SimpleRenderToTexture {
    fn init(w: &mut Worker<'_>) -> Result<Self, CoreError>
    where
        Self: Sized,
    {
        let sh_data = ShaderFiles::get_file_data(ShaderKind::Simple).unwrap();
        let shader = w
            .create_shader()
            .label("Simple shader")
            .vs_entry_point("vs_main")
            .vs_options(vec![Vertex::desc()])
            .fs_options(vec![wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            }])
            .fs_entry_point("fs_main")
            .source(sh_data)
            .build()?;

        let (vb_id, v_b_builder) = w.create_buffer_id::<Vertex>();
        let v_b = v_b_builder
            .label("Some buffer")
            .binding(0)
            .data(bytemuck::cast_slice(VERTICES))
            .usage(wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST)
            .build()?;

        let pipeline_layout = w
            .create_pipeline_layout()
            .label("Some pipeline layout")
            .build()?;
        let (p_id, pipeline_builder) = w.create_pipeline_id();
        let pipeline = pipeline_builder
            .label("Some pipeline")
            .layout(&pipeline_layout)
            .shader(&shader)
            .primitive(&wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            })
            .multisample(&wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            })
            .build()?;

        w.add_buffer(v_b);
        w.add_pipeline(pipeline);
        w.add_pipeline_layout(pipeline_layout);
        w.add_shader(shader);

        Ok(Self { p_id, vb_id })
    }

    fn update(
        &mut self,
        _w: &mut Worker<'_>,
        _event: &WindowEvent,
    ) -> std::result::Result<(), CoreError> {
        Ok(())
    }

    fn reinit(&mut self, _w: &mut Worker<'_>) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        Ok(())
    }

    fn render(&mut self, w: &mut Worker<'_>) -> Result<(), CoreError> {
        let SimpleRenderToTexture {
            ref vb_id,
            ref p_id,
            ..
        } = self;

        let pipeline = w.get_pipeline_ref(*p_id)?;
        let vb = w.get_buffer_ref(*vb_id)?;

        let view = w.texture_view()?;
        let r_p = w
            .render_pass()
            .label("Render Pass")
            .color_attachments_builder(
                ColorAttachmentBuilder::new()
                    .label("Some color attach")
                    .view(&view)
                    .ops(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    }),
            )
            .render_stage(
                0,
                RenderStage::new(&pipeline)
                    .instances(0..1)
                    .entities(0..3)
                    .vertex_buffer(&vb),
            );

        w.render(r_p)?;
        w.present()?;

        Ok(())
    }
}
