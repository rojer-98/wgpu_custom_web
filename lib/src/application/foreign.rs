#[cfg(target_arch = "wasm32")]
mod export_functions;

#[cfg(target_arch = "wasm32")]
pub use export_functions::*;

use log::info;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum UserEvent {
    Test,
}

impl UserEvent {
    pub fn on_event(&self) {
        match self {
            UserEvent::Test => info!("I am from web"),
        }
    }
}
