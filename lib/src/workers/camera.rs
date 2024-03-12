use anyhow::Result;
use cgmath::Deg;
use instant::Duration;
use winit::event::WindowEvent;

use custom_engine_components::{
    components::{
        camera::{Camera, CameraController, CameraData},
        projection::Projection,
    },
    traits::Component,
};
use custom_engine_core::{
    errors::CoreError,
    pipeline::layout::PipelineLayoutBuilder,
    render_pass::RenderStage,
    traits::{Builder, RenderWorker},
    uniform::UniformDescription,
    worker::Worker,
};

#[derive(Debug)]
pub struct CameraComponent {
    camera: Camera,
    camera_id: usize,
}

impl CameraComponent {
    pub fn init(w: &mut Worker<'_>, bind_group_binding: u32) -> Result<Self, CoreError> {
        let size = w.size();

        let projection = Projection::new(size.0, size.1, Deg(45.), 0.1, 100.);
        let controller = CameraController::new(0.5, 0.1);
        let data = CameraData::new((0.0, 5.0, 10.0), Deg(-90.0), Deg(-20.0));

        let camera = Camera::new(projection, data, controller);

        let (c_id, c_b_builder) = w.create_uniform_id();
        let c_b = c_b_builder
            .name("Uniform block")
            .entries(UniformDescription::new(
                "Camera",
                0,
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                &[camera.data()],
            ))
            .bind_group_binding(bind_group_binding)
            .build()?;

        w.add_uniform(c_b);

        Ok(Self {
            camera,
            camera_id: c_id,
        })
    }

    pub fn update(
        &mut self,
        w: &mut Worker<'_>,
        event: &WindowEvent,
        dt: Duration,
    ) -> Result<(), CoreError> {
        self.camera.update(event, dt);

        w.update_uniform(self.camera_id, "Camera", &[self.camera.data()])?;

        Ok(())
    }

    pub fn to_render_stage<'a>(
        &'a self,
        w: &'a mut Worker<'a>,
        r_s: RenderStage<'a>,
    ) -> Result<RenderStage<'a>, CoreError> {
        let c = w.get_uniform(self.camera_id)?;

        Ok(r_s.bind_group(c.get_group()))
    }

    pub fn to_pipeline_layout<'a>(
        &'a self,
        w: &'a mut Worker<'a>,
        pl_b: PipelineLayoutBuilder<'a>,
    ) -> Result<PipelineLayoutBuilder<'a>, CoreError> {
        let c = w.get_uniform(self.camera_id)?;

        Ok(pl_b.entry(c.get_layout()))
    }
}
