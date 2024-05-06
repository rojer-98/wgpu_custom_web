#[cfg(target_arch = "wasm32")]
mod export_functions;

#[cfg(target_arch = "wasm32")]
pub use export_functions::*;

use log::info;

use custom_engine_core::traits::OnEvent;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum UserEvent {
    Test,
}

impl OnEvent for UserEvent {
    fn on_event(&self) {
        match self {
            UserEvent::Test => info!("I am from web"),
        }
    }
}
