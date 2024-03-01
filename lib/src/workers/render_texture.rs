use std::{fs::read, path::Path};

use custom_engine_components::{components::camera::Camera, traits::Component};
use custom_engine_core::{
    errors::CoreError,
    render_pass::color_attachment::ColorAttachmentBuilder,
    render_pass::RenderStage,
    traits::{Builder, RenderWorker, VertexLayout},
    uniform::UniformDescription,
    worker::Worker,
};
use custom_engine_derive::VertexLayout;

use anyhow::Result;
use winit::event::WindowEvent;

use crate::files::{ShaderFiles, ShaderKind};

pub fn get_image_data<P: AsRef<Path>>(file_name: P) -> Option<Vec<u8>> {
    read(file_name).ok()
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, VertexLayout)]
#[attributes("Vertex")]
#[attributes("0 => Float32x3, 1 => Float32x2")]
struct VertexPos {
    position: [f32; 3],
    tex_coord: [f32; 2],
}

const VERTICES_POS: &[VertexPos] = &[
    //Left
    VertexPos {
        position: [-1., -1., 0.],
        tex_coord: [0., 1.],
    },
    VertexPos {
        position: [1., 1., 0.],
        tex_coord: [1., 0.],
    },
    VertexPos {
        position: [-1., 1., 0.],
        tex_coord: [0., 0.],
    },
    //Right
    VertexPos {
        position: [1., 1., 0.],
        tex_coord: [1., 0.],
    },
    VertexPos {
        position: [-1., -1., 0.],
        tex_coord: [0., 1.],
    },
    VertexPos {
        position: [1., -1., 0.],
        tex_coord: [1., 1.],
    },
];

pub struct SimpleRenderTexture {
    vb_id: usize,
    p_id: usize,
    rt_id: usize,
    c_id: usize,

    camera: Camera,
}

impl RenderWorker for SimpleRenderTexture {
    fn init(w: &mut Worker<'_>) -> Result<Self, CoreError>
    where
        Self: Sized,
    {
        let format = w.format();
        let sh_data = ShaderFiles::get_file_data(ShaderKind::Texture).unwrap();
        let shader = w
            .create_shader()
            .label("Simple shader")
            .vs_entry_point("vs_main")
            .vs_options(vec![VertexPos::desc()])
            .fs_entry_point("fs_main")
            .fs_options(vec![wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            }])
            .source(sh_data)
            .build()?;

        let camera = Camera::default();
        let (c_id, c_b_builder) = w.create_uniform_id();
        let c_b = c_b_builder
            .name("Uniform block")
            .entries(UniformDescription::new(
                "Camera",
                0,
                wgpu::ShaderStages::VERTEX,
                &camera.data(),
            ))
            .bind_group_binding(1)
            .build()?;

        let image_data = get_image_data("some.png").unwrap();
        let (rt_id, rt_builder) = w.create_render_texture_id();
        let rt = rt_builder
            .label("Render texture")
            .bytes(&image_data)
            .bind_group_binding(0)
            .sampler_desc(wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToBorder,
                address_mode_v: wgpu::AddressMode::ClampToBorder,
                address_mode_w: wgpu::AddressMode::ClampToBorder,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.,
                ..Default::default()
            })
            .view_layout_entry(wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            })
            .sampler_layout_entry(wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            })
            .build()?;
        w.load_texture(&rt);

        let bgl = rt.bind_group_layout()?;

        let (vb_id, v_b_builder) = w.create_buffer_id::<VertexPos>();
        let v_b = v_b_builder
            .label("Some buffer")
            .binding(0)
            .data(bytemuck::cast_slice(VERTICES_POS))
            .usage(wgpu::BufferUsages::VERTEX)
            .build()?;

        let pipeline_layout = w
            .create_pipeline_layout()
            .label("Some pipeline layout")
            .entries(vec![bgl, c_b.get_layout()])
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
        w.add_render_texture(rt);
        w.add_uniform(c_b);

        Ok(Self {
            rt_id,
            p_id,
            vb_id,
            c_id,

            camera,
        })
    }

    fn reinit(&mut self, _w: &mut Worker<'_>) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        Ok(())
    }

    fn update(&mut self, w: &mut Worker<'_>, event: &WindowEvent) -> Result<(), CoreError> {
        self.camera.update(event);
        w.update_uniform(self.c_id, "Camera", &self.camera.data())?;

        Ok(())
    }

    fn render(&mut self, w: &mut Worker<'_>) -> Result<(), CoreError> {
        let SimpleRenderTexture {
            vb_id,
            p_id,
            rt_id,
            c_id,
            ..
        } = self;

        let pipeline = w.get_pipeline_ref(*p_id)?;
        let vb = w.get_buffer_ref(*vb_id)?;
        let rt = w.get_render_texture_ref(*rt_id)?;
        let c = w.get_uniform_ref(*c_id)?;

        let bg_t = rt.bind_group()?;
        let bg_c = c.get_group();

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
                .instances(0..6)
                .entities(0..6)
                .vertex_buffer(&vb)
                .bind_groups(vec![bg_t, bg_c]),
        );

        w.render(r_p)?;
        w.present()?;

        Ok(())
    }
}
