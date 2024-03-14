pub(crate) mod controller;
pub(crate) mod data;
pub(crate) mod projection;

use anyhow::Result;
use cgmath::{Deg, Matrix, SquareMatrix};
use instant::Duration;
use winit::event::WindowEvent;

use custom_engine_core::{
    errors::CoreError, traits::Builder, uniform::UniformDescription, worker::Worker,
};

use crate::{
    components::camera::{controller::CameraController, data::CameraData, projection::Projection},
    traits::Component,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraRaw {
    view_position: [f32; 4],
    view: [[f32; 4]; 4],
    view_proj: [[f32; 4]; 4],
    inv_proj: [[f32; 4]; 4],
    inv_view: [[f32; 4]; 4],
}

#[derive(Debug)]
pub struct CameraInner {
    data: CameraData,
    controller: CameraController,
    projection: Projection,
}

impl CameraInner {
    pub fn new(projection: Projection, data: CameraData, controller: CameraController) -> Self {
        Self {
            projection,
            data,
            controller,
        }
    }
}

impl Component<CameraRaw> for CameraInner {
    fn data(&self) -> CameraRaw {
        let proj = self.projection.matrix();
        let view = self.data.matrix();
        let view_proj = proj * view;

        let view_position = self.data.position.to_homogeneous().into();
        let inv_proj = proj.invert().unwrap().into();
        let inv_view = view.transpose().into();
        let view_proj = view_proj.into();
        let view = view.into();

        CameraRaw {
            view_position,
            inv_view,
            inv_proj,
            view_proj,
            view,
        }
    }

    fn update(&mut self, event: &WindowEvent, dt: Duration) {
        if self.controller.process_events(event) {
            self.data.update(&mut self.controller, dt);
        }
    }
}

#[derive(Debug)]
pub struct Camera {
    pub camera_id: usize,
    camera: CameraInner,
}

impl Camera {
    pub fn init(w: &mut Worker<'_>, bind_group_binding: u32) -> Result<Self, CoreError> {
        let size = w.size();

        let projection = Projection::new(size.0, size.1, Deg(45.), 0.1, 100.);
        let controller = CameraController::new(0.5, 0.1);
        let data = CameraData::new((0.0, 5.0, 10.0), Deg(-90.0), Deg(-20.0));

        let camera = CameraInner::new(projection, data, controller);

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
}
