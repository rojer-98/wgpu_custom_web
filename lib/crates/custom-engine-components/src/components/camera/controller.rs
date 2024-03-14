use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{Key, NamedKey},
};

#[derive(Debug, Default)]
pub struct CameraController {
    pub amount_left: f32,
    pub amount_right: f32,
    pub amount_forward: f32,
    pub amount_backward: f32,
    pub amount_up: f32,
    pub amount_down: f32,

    pub rotate_horizontal: f32,
    pub rotate_vertical: f32,

    pub scroll: f32,
    pub speed: f32,
    pub sensitivity: f32,

    pub old_mouse_position: (f64, f64),
    pub mouse_pressed: bool,
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

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
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
