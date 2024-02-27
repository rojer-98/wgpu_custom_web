mod config;
mod errors;
mod files;
mod runner;
mod workers;

use runner::EngineRunner;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
const CONFIG: &'static str = include_str!("../assets/config.toml");

#[cfg(not(target_arch = "wasm32"))]
pub fn run() {
    EngineRunner::new(CONFIG)
        .expect("Init conifg error: ")
        .logger()
        .expect("Init logger error: ")
        .run()
        .expect("Render loop error: ");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run(config: String) {
    EngineRunner::new(config)
        .expect("Init conifg error: ")
        .logger()
        .expect("Init logger error: ")
        .run()
        .expect("Render loop error: ");
}
