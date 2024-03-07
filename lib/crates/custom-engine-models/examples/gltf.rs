use std::{path::PathBuf, str::FromStr};

use custom_engine_models::gltf::load;

fn main() {
    let mut path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    path.push("data");
    path.push("glTFPotOfCoals.glb");

    if let Some((r, s)) = load(path.to_str().unwrap(), 0) {
        println!("{r:?}\n{s:?}");
    }
}
