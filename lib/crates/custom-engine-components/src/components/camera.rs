use std::{f32::consts::FRAC_PI_2, time::Duration};

use cgmath::{InnerSpace, Matrix, Matrix4, Point3, Rad, SquareMatrix, Vector3};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{Key, NamedKey},
};

use crate::{components::projection::Projection, traits::Component};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

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
pub struct Camera {
    data: CameraData,
    controller: CameraController,
    projection: Projection,
}

impl Camera {
    pub fn new(projection: Projection, data: CameraData, controller: CameraController) -> Self {
        Self {
            projection,
            data,
            controller,
        }
    }
}

impl Component<CameraRaw> for Camera {
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
pub struct CameraData {
    position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl CameraData {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    #[inline]
    pub fn matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }

    fn update(&mut self, controller: &mut CameraController, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = self.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        self.position += forward
            * (controller.amount_forward - controller.amount_backward)
            * controller.speed
            * dt;
        self.position +=
            right * (controller.amount_right - controller.amount_left) * controller.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = self.pitch.0.sin_cos();
        let scrollward =
            Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        self.position +=
            scrollward * controller.scroll * controller.speed * controller.sensitivity * dt;
        controller.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        self.position.y += (controller.amount_up - controller.amount_down) * controller.speed * dt;

        // Rotate
        self.yaw += Rad(controller.rotate_horizontal) * controller.sensitivity * dt;
        self.pitch += Rad(-controller.rotate_vertical) * controller.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        controller.rotate_horizontal = 0.0;
        controller.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if self.pitch < -Rad(SAFE_FRAC_PI_2) {
            self.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if self.pitch > Rad(SAFE_FRAC_PI_2) {
            self.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}

#[derive(Debug, Default)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,

    rotate_horizontal: f32,
    rotate_vertical: f32,

    scroll: f32,
    speed: f32,
    sensitivity: f32,

    old_mouse_position: (f64, f64),
    mouse_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.,
            amount_right: 0.,
            amount_forward: 0.,
            amount_backward: 0.,
            amount_up: 0.,
            amount_down: 0.,
            rotate_horizontal: 0.,
            rotate_vertical: 0.,
            scroll: 0.,
            speed,
            sensitivity,
            old_mouse_position: (0., 0.),
            mouse_pressed: false,
        }
    }

    pub fn process_keyboard(&mut self, key: Key, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            Key::Named(NamedKey::ArrowUp) => {
                self.amount_forward = amount;
                true
            }
            Key::Named(NamedKey::ArrowDown) => {
                self.amount_backward = amount;
                true
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.amount_left = amount;
                true
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.amount_right = amount;
                true
            }
            Key::Named(NamedKey::Space) => {
                self.amount_up = amount;
                true
            }
            Key::Named(NamedKey::Shift) => {
                self.amount_down = amount;
                true
            }
            Key::Character(s) => match s.as_str() {
                "W" | "w" => {
                    self.amount_forward = amount;
                    true
                }
                "S" | "s" => {
                    self.amount_backward = amount;
                    true
                }
                "A" | "a" => {
                    self.amount_left = amount;
                    true
                }
                "D" | "d" => {
                    self.amount_right = amount;
                    true
                }

                _ => false,
            },
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) -> bool {
        self.rotate_horizontal += mouse_dx as f32;
        self.rotate_vertical += mouse_dy as f32;

        true
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) -> bool {
        self.scroll = match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => -scroll * 3.5,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => -*scroll as f32,
        };

        true
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                let keycode = event.logical_key.clone();
                let state = event.state;

                self.process_keyboard(keycode, state)
            }
            WindowEvent::MouseWheel { delta, .. } => self.process_scroll(delta),
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_pressed = state.is_pressed();
                true
            }
            WindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => {
                let (x, y) = (*x as f64, *y as f64);
                let old_mouse_position = self.old_mouse_position.clone();

                self.old_mouse_position = (x, y);
                if self.mouse_pressed {
                    let (dx, dy) = (x - old_mouse_position.0, y - old_mouse_position.1);

                    self.process_mouse(dx / 100., dy / 100.)
                } else {
                    false
                }
            }

            _ => false,
        }
    }
}
