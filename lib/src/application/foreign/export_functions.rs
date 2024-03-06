use wasm_bindgen::prelude::*;

use crate::{application::foreign::UserEvent, errors::EngineError, runner::EVENT_LOOP_PROXY};

macro_rules! send_event {
    ($event:expr) => {
        unsafe {
            if let Some(elp) = EVENT_LOOP_PROXY.as_ref() {
                if let Err(e) = elp.send_event($event) {
                    log::error!("{e}");
                }
            }
        }
    };
}

#[wasm_bindgen]
pub fn user_event_action() {
    send_event!(UserEvent::Test);
}
