mod foreign;
mod key_bindings;

use instant::{Duration, Instant};
use winit::{dpi::PhysicalPosition, event::ElementState};

pub enum ClickType {
    Simple,
    Double,
    Triple,
}

pub struct AppState {
    pub(crate) cursor_position: PhysicalPosition<f64>,

    pub(crate) click_state: ElementState,
    pub(crate) click_position: PhysicalPosition<f64>,

    pub(crate) last_click_timestamp: Option<Instant>,
    is_double_click: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            cursor_position: PhysicalPosition::new(0., 0.),
            click_state: ElementState::Released,
            click_position: PhysicalPosition::new(0., 0.),
            last_click_timestamp: None,
            is_double_click: false,
        }
    }

    pub fn clicked(&mut self) {
        if self.last_click_timestamp.is_none() {
            self.last_click_timestamp = Some(Instant::now());
        } else {
            let lct = self.last_click_timestamp.take().unwrap();
            let elapsed = lct.elapsed();

            self.is_double_click = elapsed.saturating_sub(Duration::from_millis(400)).is_zero();
        }

        self.click_position = self.cursor_position;
    }

    pub fn is_double_click(&mut self) -> bool {
        self.is_double_click
    }
}
