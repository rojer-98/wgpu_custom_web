use custom_engine_components::object::triangle::{Triangle, Triangles};
use custom_engine_core::{
    errors::CoreError,
    render_pass::color_attachment::ColorAttachmentBuilder,
    render_pass::RenderStage,
    traits::{Builder, RenderWorker},
    uniform::UniformDescription,
    worker::Worker,
};

use anyhow::Result;
use cgmath::Vector3;
use winit::event::WindowEvent;

use crate::{
    application::AppState,
    errors::EngineError,
    files::{ShaderFiles, ShaderKind},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct UData {
    pub size: [f32; 4],
}

pub struct SimpleRender {
    shift: f32,
    counter: usize,
    data: Triangles,

    vb_id: usize,
    p_id: usize,
    c_id: usize,
}

impl SimpleRender {
    pub fn click(&mut self, w: &mut Worker<'_>, app: &AppState) -> Result<(), EngineError> {
        self.data.click(Vector3::new(
            app.click_position.x as f32,
            app.click_position.y as f32,
            0.,
        ));

        w.update_buffer(self.vb_id, 0, &self.data.to_data())?;

        Ok(())
    }

    pub fn move_to(&mut self, w: &mut Worker<'_>, diff: (f64, f64)) -> Result<(), EngineError> {
        self.data
            .move_to(Vector3::new(diff.0 as f32, diff.1 as f32, 0.));

        w.update_buffer(self.vb_id, 0, &self.data.to_data())?;

        Ok(())
    }
}

impl RenderWorker for SimpleRender {
    fn init(w: &mut Worker<'_>) -> Result<Self, CoreError>
    where
        Self: Sized,
    {
        let shift = 0.001;
        let first_triangle = Triangle::new(
            [
                [1000.0, 150.5, 0.0],
                [300.5, 500.5, 0.0],
                [1000.5, 200.5, 0.0],
            ],
            [1.0, 0.0, 0.0],
        );
        let data: Triangles = vec![first_triangle].into();

        let format = w.format();
        let sh_data = ShaderFiles::get_file_data(ShaderKind::Simple).unwrap();
        let shader = w
            .create_shader()
            .label("Simple shader")
            .vs_entry_point("vs_main")
            .vs_options(vec![Triangle::desc()])
            .fs_options(vec![wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::OVER,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            }])
            .fs_entry_point("fs_main")
            .source(sh_data)
            .build()?;

        let (vb_id, v_b_builder) = w.create_buffer_id();
        let v_b = v_b_builder
            .label("Some buffer")
            .binding(0)
            .size(1024)
            .data(&data.to_data())
            .usage(wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST)
            .build()?;

        let size = w.size();
        let c_data = UData {
            size: [size.0 as f32, size.1 as f32, 0., 0.],
        };
        let (c_id, c_b) = w.create_uniform_id();
        let c = c_b
            .name("Uniform block")
            .bind_group_binding(0)
            .entries(UniformDescription::new(
                "Controls",
                0,
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                &[c_data],
            ))
            .build()?;

        let pipeline_layout = w
            .create_pipeline_layout()
            .entry(c.get_layout())
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
                cull_mode: None,
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

        w.add_uniform(c);
        w.add_buffer(v_b);
        w.add_pipeline(pipeline);
        w.add_pipeline_layout(pipeline_layout);
        w.add_shader(shader);

        Ok(Self {
            p_id,
            vb_id,
            c_id,
            data,
            shift,
            counter: 0,
        })
    }

    fn reinit(&mut self, _w: &mut Worker<'_>) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        Ok(())
    }

    fn update(&mut self, _w: &mut Worker<'_>, _event: &WindowEvent) -> Result<(), CoreError> {
        Ok(())
    }

    fn resize(&mut self, w: &mut Worker<'_>) -> std::prelude::v1::Result<(), CoreError> {
        let size = w.size();
        let c_data = UData {
            size: [size.0 as f32, size.1 as f32, 0., 0.],
        };

        w.update_uniform(self.c_id, "Controls", &[c_data])?;

        Ok(())
    }

    fn render(&mut self, w: &mut Worker<'_>) -> Result<(), CoreError> {
        let SimpleRender {
            vb_id, p_id, c_id, ..
        } = self;

        let pipeline = w.get_pipeline_ref(*p_id)?;
        let vb = w.get_buffer_ref(*vb_id)?;
        let c = w.get_uniform_ref(*c_id)?;

        let view = w.texture_view()?;
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
                .entities(0..16)
                .bind_groups(vec![c.get_group()])
                .vertex_buffer(&vb),
        );

        w.render(r_p)?;
        w.present()?;

        if self.counter < 4 {
            self.counter += 1;
            self.data.push(Triangle::new(
                [
                    [1000.0 - self.shift, 150.5 + self.shift, 0.0],
                    [300.5 - self.shift, 500.5 + self.shift, 0.0],
                    [1000.5 - self.shift, 200.5 + self.shift, 0.0],
                ],
                [1.0, 0.0, 0.0],
            ));
            self.shift += 100.;

            w.update_buffer(*vb_id, 0, &self.data.to_data())?;
        }

        Ok(())
    }
}
