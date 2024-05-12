use anyhow::Result;

use custom_engine_core::{
    errors::CoreError,
    render_pass::color_attachment::ColorAttachmentBuilder,
    render_pass::RenderStage,
    storage::{StorageDescription, StorageKind},
    traits::{Builder, RenderWorker, VertexLayout},
    worker::Worker,
};
use custom_engine_derive::VertexLayout;
use pollster::block_on;

use crate::files::{ShaderFiles, ShaderKind};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, VertexLayout)]
#[attributes("Vertex")]
#[attributes("0 => Float32x3, 1 => Float32x3")]
struct Vertex {
    position: [f32; 3],
    _pad1: f32,
    color: [f32; 3],
    _pad2: f32,
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        _pad1: 0.0,
        color: [1.0, 0.0, 0.0],
        _pad2: 0.0,
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        _pad1: 0.0,
        color: [0.0, 1.0, 0.0],
        _pad2: 0.0,
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        _pad1: 0.0,
        color: [0.0, 0.0, 1.0],
        _pad2: 0.0,
    },
];

#[derive(Debug, Default)]
pub struct SimpleCustomRender {
    vb_id: usize,
    s_id: usize,
    p_id: usize,

    counter: f32,
}

impl RenderWorker for SimpleCustomRender {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            ..Default::default()
        }
    }

    fn init(&mut self, w: &mut Worker<'_>) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        let sh_data = ShaderFiles::get_file_data(ShaderKind::Custom).unwrap();
        let shader = w
            .create_shader()
            .label("Simple shader")
            .vs_entry_point("vs_main")
            .vs_options(vec![Vertex::desc()])
            .fs_entry_point("fs_main")
            .fs_options(vec![wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            }])
            .source(sh_data)
            .build()?;

        let (s_id, s_builder) = w.create_storage_id();
        let s = s_builder
            .entries(StorageDescription::new(
                "Storage",
                0,
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                StorageKind::Buffer { read_only: false },
                &vec![
                    Vertex {
                        position: [0.0, 0.0, 0.0],
                        _pad1: 0.0,
                        color: [0.0, 0.0, 0.0],
                        _pad2: 0.0,
                    };
                    10
                ],
            ))
            .bind_group_binding(0)
            .build()?;

        let (vb_id, v_b_builder) = w.create_buffer_id();
        let v_b = v_b_builder
            .label("Some buffer")
            .binding(0)
            .data(VERTICES)
            .usage(wgpu::BufferUsages::VERTEX)
            .build()?;

        let pipeline_layout = w
            .create_pipeline_layout()
            .label("Some pipeline layout")
            .entry(s.get_layout())
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
        w.add_storage(s);
        w.add_pipeline(pipeline);
        w.add_pipeline_layout(pipeline_layout);
        w.add_shader(shader);

        *self = Self {
            p_id,
            vb_id,
            s_id,
            counter: 0.0,
        };

        Ok(())
    }

    fn render(&mut self, w: &mut Worker<'_>) -> Result<(), CoreError> {
        let SimpleCustomRender {
            vb_id, p_id, s_id, ..
        } = self;

        let pipeline = w.get_pipeline_ref(*p_id)?;
        let vb = w.get_buffer_ref(*vb_id)?;
        let s = w.get_storage_ref(*s_id)?;

        let view = w.view_surface()?;
        let r_p = w.render_pass().label("Render Pass").render_stage(
            0,
            RenderStage::new(&pipeline)
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
                .instances(0..1)
                .entities(0..3)
                .vertex_buffer(&vb)
                .bind_groups(vec![s.get_group()]),
        );

        w.render(r_p)?;
        block_on(async { w.present().await })?;

        let out = block_on(async { w.read_storage_buffer::<Vertex>(s.id, "Storage").await })?;

        w.update_storage(
            s.id,
            "Storage",
            &out.into_iter()
                .map(|mut v| {
                    v.color[2] *= self.counter;
                    v.position[0] *= self.counter;

                    v
                })
                .collect::<Vec<_>>(),
        )?;
        self.counter += 0.0001;

        Ok(())
    }
}
