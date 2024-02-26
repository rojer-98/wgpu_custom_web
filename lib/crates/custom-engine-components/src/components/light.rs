use cgmath::Vector3;
use winit::{
    event::WindowEvent,
    keyboard::{Key, NamedKey},
};

use crate::traits::Component;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightRaw {
    position: [f32; 3],
    _padding: u32,
    color: [f32; 3],
    _padding2: u32,
}

#[derive(Debug)]
pub struct Light {
    controller: LightController,
    data: LightData,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            controller: Default::default(),
            data: Default::default(),
        }
    }
}

impl Component<1, LightRaw> for Light {
    fn data(&self) -> [LightRaw; 1] {
        [LightRaw {
            position: self.data.position.into(),
            _padding: 0,
            color: self.data.color.into(),
            _padding2: 0,
        }; 1]
    }

    fn update(&mut self, event: &WindowEvent) {
        if self.controller.process_events(event) {
            self.data.update_camera(&self.controller);
            self.controller.reset();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LightData {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
}

impl Default for LightData {
    fn default() -> Self {
        Self {
            position: Vector3 {
                x: 2.,
                y: 2.,
                z: 2.,
            },
            color: Vector3 {
                x: 1.,
                y: 1.,
                z: 1.,
            },
        }
    }
}

impl LightData {
    fn update_camera(&mut self, controller: &LightController) {
        let old_position = self.position;
        let shift_vec = if controller.is_forward_pressed {
            Vector3::new(0., 0., -0.05)
        } else if controller.is_backward_pressed {
            Vector3::new(0., 0., 0.05)
        } else if controller.is_left_pressed {
            Vector3::new(-0.05, 0., 0.)
        } else if controller.is_right_pressed {
            Vector3::new(0.05, 0., 0.)
        } else {
            Vector3::new(0., 0., 0.)
        };

        self.position = shift_vec + old_position;
    }
}

#[derive(Debug)]
struct LightController {
    speed: f32,

    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl Default for LightController {
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

impl LightController {
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
