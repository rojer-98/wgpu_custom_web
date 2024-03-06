use std::path::PathBuf;

use log::info;

use custom_engine_models::gltf::load;

fn main() {
    let path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    if let Some(g) = load(path, 0) {
        info!("{g:?}");
    }
}
