mod application;
mod config;
mod errors;
mod files;
mod runner;
mod workers;

use runner::EngineRunner;

#[cfg(not(target_arch = "wasm32"))]
pub fn run() {
    EngineRunner::new(include_str!("../assets/config.toml"))
        .expect("Init conifg error: ")
        .logger()
        .expect("Init logger error: ")
        .run()
        .expect("Render loop error: ");
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(catch)]
pub fn run() {
    use custom_engine_utils::get_string;

    let config = get_string("./assets/config.toml").expect("Config not found");

    EngineRunner::new(&config)
        .expect("Init conifg error: ")
        .logger()
        .expect("Init logger error: ")
        .run()
        .expect("Render loop error: ");
}
