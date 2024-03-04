mod foreign;
mod key_bindings;

use winit::{dpi::PhysicalPosition, event::ElementState};

pub struct AppState {
    pub(crate) cursor_position: PhysicalPosition<f64>,

    pub(crate) click_state: ElementState,
    pub(crate) click_position: PhysicalPosition<f64>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            cursor_position: PhysicalPosition::new(0., 0.),
            click_state: ElementState::Released,
            click_position: PhysicalPosition::new(0., 0.),
        }
    }
}
