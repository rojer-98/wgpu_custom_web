use std::collections::HashMap;

use custom_engine_components::{
    components::{camera::Camera, light::Light},
    traits::Component,
};
use custom_engine_core::{
    errors::CoreError,
    instance::Instances,
    model::Model,
    render_pass::RenderStage,
    render_pass::{
        color_attachment::ColorAttachmentBuilder, depth_stencil::DepthStencilAttachmentBuilder,
    },
    texture::TextureKind,
    traits::{Builder, RenderWorker},
    uniform::UniformDescription,
    worker::Worker,
};
use custom_engine_models::{gltf::GltfFile, obj::ObjFile};

use anyhow::Result;
use winit::event::WindowEvent;

use crate::files::{ShaderFiles, ShaderKind};

const NUM_INSTANCES_PER_ROW: u32 = 10;
const SPACE_BETWEEN: f32 = 3.0;

pub struct SimpleModelRender {
    sh_id: usize,
    pl_id: usize,
    p_id: usize,
    m_id: usize,
    vb_id: usize,

    c_id: usize,

    hdr_t_id: usize,
    hdr_p_id: usize,
    hdr_sh_id: usize,
    hdr_pl_id: usize,

    camera: Camera,
    light: Light,
    size: (u32, u32),
}

impl RenderWorker for SimpleModelRender {
    async fn init(w: &mut Worker<'_>) -> Result<Self, CoreError>
    where
        Self: Sized,
    {
        let obj_file = ObjFile::new("./assets/models/cube/cube.obj").await?;
        let gltf_file = GltfFile::new("./assets/models/toycar/ToyCar.glb").await?;

        let (m_id, m_builder) = w.create_model_id();
        let m = m_builder
            .obj_file(obj_file)
            .diffuse_view_binding(0)
            .diffuse_sampler_binding(1)
            .diffuse_format(TextureKind::HDR)
            .normal_view_binding(2)
            .normal_sampler_binding(3)
            .normal_format(TextureKind::HDR)
            .mesh_vertex_binding(0)
            .build()?;
        w.load_model(&m);

        let bgl = m.bind_group_layout();

        let instances = Instances::new(SPACE_BETWEEN, NUM_INSTANCES_PER_ROW).data();
        let (vb_id, v_b_builder) = w.create_buffer_id();
        let v_b = v_b_builder
            .label("Some buffer")
            .binding(1)
            .data(&instances)
            .usage(wgpu::BufferUsages::VERTEX)
            .build()?;

        let camera = Camera::default();
        let light = Light::default();

        let (c_id, c_b_builder) = w.create_uniform_id();
        let c_b = c_b_builder
            .name("Uniform block")
            .entries(UniformDescription::new(
                "Camera",
                0,
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                &[camera.data()],
            ))
            .entries(UniformDescription::new(
                "Light",
                1,
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                &[light.data()],
            ))
            .bind_group_binding(1)
            .build()?;

        let (sh_id, v_shader_builder) = w.create_shader_id();
        let format = w.format();

        let sh_data = ShaderFiles::get_file_data(ShaderKind::Model).unwrap();
        let shader = v_shader_builder
            .label("Simple shader")
            .vs_entry_point("vs_main")
            .vs_options(vec![
                Model::get_buffer_layout(),
                Instances::get_buffer_layout(),
            ])
            .fs_options(vec![wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            }])
            .fs_entry_point("fs_main")
            .source(sh_data)
            .build()?;

        let (pl_id, pipeline_layout_builder) = w.create_pipeline_layout_id();
        let pipeline_layout = pipeline_layout_builder
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
            .depth_stencil(&wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(),     // 2.
                bias: wgpu::DepthBiasState::default(),
            })
            .multisample(&wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            })
            .build()?;

        let (hdr_sh_id, hdr_sh_builder) = w.create_shader_id();
        let sh_data = ShaderFiles::get_file_data(ShaderKind::HDR).unwrap();
        let hdr_sh = hdr_sh_builder
            .label("HDR shader")
            .vs_entry_point("vs_main")
            .fs_options(vec![wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            }])
            .fs_entry_point("fs_main")
            .source(sh_data)
            .build()?;

        let size = w.size();
        let (hdr_t_id, hdr_t_builder) = w.create_render_texture_id();
        let hdr_t = hdr_t_builder
            .label("HDR texture")
            .format(format)
            .usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT)
            .texture_size(size)
            .view_layout_entry(wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            })
            .sampler_layout_entry(wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            })
            .sampler_desc(wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            })
            .bind_group_binding(0)
            .is_sampler(true)
            .build()?;

        let hdr_bgl = hdr_t.bind_group_layout()?;

        let (hdr_pl_id, hdr_pl_builder) = w.create_pipeline_layout_id();
        let hdr_pl = hdr_pl_builder
            .label("HDR pipeline layout")
            .entry(hdr_bgl)
            .build()?;

        let (hdr_p_id, hdr_p_builder) = w.create_pipeline_id();
        let hdr_p = hdr_p_builder
            .layout(&hdr_pl)
            .shader(&hdr_sh)
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

        w.add_pipeline(pipeline);
        w.add_pipeline_layout(pipeline_layout);
        w.add_shader(shader);
        w.add_model(m);
        w.add_buffer(v_b);
        w.add_uniform(c_b);

        w.add_render_texture(hdr_t);
        w.add_shader(hdr_sh);
        w.add_pipeline_layout(hdr_pl);
        w.add_pipeline(hdr_p);

        Ok(Self {
            c_id,
            pl_id,
            p_id,
            sh_id,
            m_id,
            vb_id,

            hdr_t_id,
            hdr_p_id,
            hdr_sh_id,
            hdr_pl_id,

            light,
            camera,
            size,
        })
    }

    async fn reinit(&mut self, w: &mut Worker<'_>) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        let SimpleModelRender {
            pl_id, sh_id, p_id, ..
        } = self;

        let pipeline_layout = w.get_pipeline_layout_ref(*pl_id)?;

        let format = w.format();
        let sh_data = ShaderFiles::get_file_data(ShaderKind::Model).unwrap();
        let shader = w
            .create_shader()
            .label("Simple shader")
            .vs_entry_point("vs_main")
            .vs_options(vec![
                Model::get_buffer_layout(),
                Instances::get_buffer_layout(),
            ])
            .fs_options(vec![wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            }])
            .fs_entry_point("fs_main")
            .source(sh_data)
            .build()?;

        let pipeline = w
            .create_pipeline()
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
            .depth_stencil(&wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
            .multisample(&wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            })
            .build()?;

        w.replace_pipeline(*p_id, pipeline)?;
        w.replace_shader(*sh_id, shader)?;

        Ok(())
    }

    async fn render(&mut self, w: &mut Worker<'_>) -> Result<(), CoreError> {
        let SimpleModelRender {
            m_id,
            p_id,
            vb_id,
            c_id,
            hdr_p_id,
            hdr_t_id,
            size,
            ..
        } = self;

        let pipeline = w.get_pipeline_ref(*p_id)?;
        let m = w.get_model_ref(*m_id)?;
        let vb = w.get_buffer_ref(*vb_id)?;
        let c = w.get_uniform_ref(*c_id)?;

        let hdr_pipeline = w.get_pipeline_ref(*hdr_p_id)?;
        let hdr_texture = w.get_render_texture_ref(*hdr_t_id)?;

        let hdr_bind_group = hdr_texture.bind_group()?;
        let hdr_t_view = hdr_texture.view();

        let d_t = w
            .create_depth_texture()
            .label("Depth Texture")
            .texture_size(*size)
            .build()?;
        let d_t_view = d_t.view;

        let view = w.texture_view()?;
        let r_p = w
            .render_pass()
            .label("Render Pass")
            .render_stage(
                0,
                RenderStage::new(&pipeline)
                    .depth_stencil_builder(
                        DepthStencilAttachmentBuilder::new()
                            .label("Some depth attach")
                            .view(&d_t_view)
                            .depth_ops(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                    )
                    .color_attachments_builder(
                        ColorAttachmentBuilder::new()
                            .label("Some color attach")
                            .view(hdr_t_view)
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
                    .entities(0..1)
                    .instances(0..30)
                    .vertex_buffer(&vb)
                    .bind_groups(vec![c.get_group()])
                    .model(&m),
            )
            .render_stage(
                1,
                RenderStage::new(&hdr_pipeline)
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
                    .bind_groups(vec![hdr_bind_group])
                    .instances(0..1)
                    .entities(0..3),
            );

        w.render(r_p)?;
        w.present().await?;

        Ok(())
    }

    fn update(&mut self, w: &mut Worker<'_>, event: &WindowEvent) -> Result<(), CoreError> {
        self.camera.update(event);
        self.light.update(event);

        w.update_uniform(self.c_id, "Camera", &[self.camera.data()])?;
        w.update_uniform(self.c_id, "Light", &[self.light.data()])?;

        Ok(())
    }
}
