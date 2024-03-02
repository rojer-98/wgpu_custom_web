mod foreign;
mod key_bindings;

use anyhow::Result;

use winit::{dpi::PhysicalPosition, event_loop::EventLoop};

use crate::errors::EngineError;

pub struct Application {
    event_loop: EventLoop<()>,

    cursor_position: PhysicalPosition<f64>,
}

impl Application {
    pub fn new(event_loop: EventLoop<()>) -> Self {
        Self {
            event_loop,
            cursor_position: PhysicalPosition::new(0., 0.),
        }
    }

    pub fn start(self) -> Result<(), EngineError> {
        Ok(())
    }
}
