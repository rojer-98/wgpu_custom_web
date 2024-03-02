mod foreign;
mod key_bindings;

use winit::dpi::PhysicalPosition;
pub struct AppState {
    pub(crate) cursor_position: PhysicalPosition<f64>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            cursor_position: PhysicalPosition::new(0., 0.),
        }
    }
}
