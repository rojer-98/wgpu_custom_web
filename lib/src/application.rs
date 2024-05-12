pub mod foreign;
pub mod key_bindings;

use instant::{Duration, Instant};
use winit::{dpi::PhysicalPosition, event::ElementState};

use custom_engine_core::traits::EventHandler;

use crate::workers::model::SimpleModelRender;

#[derive(Debug)]
pub enum ClickType {
    Simple,
    Double,
    Triple,
}

#[derive(Debug)]
pub struct AppState {
    pub(crate) cursor_position: PhysicalPosition<f64>,

    pub(crate) click_state: ElementState,
    pub(crate) click_position: PhysicalPosition<f64>,

    pub(crate) last_click_timestamp: Option<Instant>,
    is_double_click: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            cursor_position: PhysicalPosition::new(0., 0.),
            click_state: ElementState::Released,
            click_position: PhysicalPosition::new(0., 0.),
            last_click_timestamp: None,
            is_double_click: false,
        }
    }
}

impl AppState {
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

/*

 -        // Mouse
-        // WindowEvent::CursorMoved { position, .. } => {
-        //     if app_state.click_state.is_pressed() {
-        //         let diff = (
-        //             position.x - app_state.cursor_position.x,
-        //             position.y - app_state.cursor_position.y,
-        //         );
-        //         r.move_to(&mut worker_surface, diff).unwrap();
-        //     }
-
-        //     app_state.cursor_position = *position;
-        // }
-        // WindowEvent::MouseInput { state, .. } => {
-        //     if state.is_pressed() {
-        //         app_state.clicked();
-
-        //         r.click(&mut worker_surface, &app_state).unwrap();
-        //     }
-
-        //     app_state.click_state = *state;
-        // }
-        _ => {}
-    }
*/

impl EventHandler<SimpleModelRender> for AppState {}
