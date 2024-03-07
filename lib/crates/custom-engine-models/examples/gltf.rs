use std::{path::PathBuf, str::FromStr};

use custom_engine_models::gltf::GltfFile;

fn main() {
    let mut path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    path.push("data");
    path.push("glTFPotOfCoals.glb");

    let mut gltf_file = GltfFile::new(path.to_str().unwrap()).unwrap();

    if let Ok(s) = gltf_file.scene(0) {
        println!("{s:?}");
    }
}
