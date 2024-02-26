use cgmath::{perspective, Deg, InnerSpace, Matrix4, Point3, Vector3, Vector4};
use derive_more::Constructor;
use winit::{
    event::WindowEvent,
    keyboard::{Key, NamedKey},
};

use crate::traits::Component;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraRaw {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

#[derive(Debug, Constructor)]
pub struct Camera {
    data: CameraData,
    controller: CameraController,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            controller: Default::default(),
            data: Default::default(),
        }
    }
}

impl Component<1, CameraRaw> for Camera {
    fn data(&self) -> [CameraRaw; 1] {
        const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 1.0,
        );

        let view = self.data.view();
        let proj = self.data.projection();

        let view_proj: [[f32; 4]; 4] = (OPENGL_TO_WGPU_MATRIX * proj * view).into();
        let view_position: [f32; 4] = self.data.position().into();

        [CameraRaw {
            view_position,
            view_proj,
        }; 1]
    }

    fn update(&mut self, event: &WindowEvent) {
        if self.controller.process_events(event) {
            self.data.update_camera(&self.controller);
            self.controller.reset();
        }
    }
}

#[derive(Debug, Constructor)]
struct CameraData {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            eye: (0.0, 5.0, -30.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: 1.,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}

impl CameraData {
    #[inline(always)]
    fn projection(&self) -> Matrix4<f32> {
        perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar)
    }

    #[inline(always)]
    fn view(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.eye, self.target, self.up)
    }

    #[inline(always)]
    fn position(&self) -> Vector4<f32> {
        self.eye.to_homogeneous()
    }

    fn update_camera(&mut self, controller: &CameraController) {
        let forward = self.target - self.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        if controller.is_forward_pressed && forward_mag > controller.speed {
            self.eye += forward_norm * controller.speed;
        }
        if controller.is_backward_pressed {
            self.eye -= forward_norm * controller.speed;
        }

        let right = forward_norm.cross(self.up);
        let forward = self.target - self.eye;
        let forward_mag = forward.magnitude();

        if controller.is_right_pressed {
            self.eye = self.target - (forward + right * controller.speed).normalize() * forward_mag;
        }
        if controller.is_left_pressed {
            self.eye = self.target - (forward - right * controller.speed).normalize() * forward_mag;
        }
    }
}

#[derive(Debug)]
struct CameraController {
    speed: f32,

    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            speed: 0.2,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            ..Default::default()
        }
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                let keycode = event.logical_key.clone();
                let is_pressed = event.state.is_pressed();

                match keycode {
                    Key::Named(NamedKey::ArrowUp) => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    Key::Named(NamedKey::ArrowLeft) => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    Key::Named(NamedKey::ArrowDown) => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn reset(&mut self) {
        *self = Self {
            speed: self.speed,
            ..Default::default()
        };
    }
}
