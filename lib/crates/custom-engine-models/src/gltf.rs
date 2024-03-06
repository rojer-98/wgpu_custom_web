mod camera;
mod document;
mod material;
mod mesh;
mod node;
mod primitive;
mod root;
mod scene;
mod texture;

pub use camera::*;
pub use document::*;
pub use material::*;
pub use mesh::*;
pub use node::*;
pub use primitive::*;
pub use root::*;
pub use scene::*;
pub use texture::*;

use std::path::Path;

use gltf::{
    buffer::Data as GltfBufferData, camera::Camera as GltfCamera, image::Data as GltfImageData,
    material::Material as GltfMaterial, mesh::Mesh as GltfMesh, scene::Scene as GltfScene,
    texture::Texture as GltfTexture, Document as GltfDocument, Primitive as GltfPrimitive,
};
use log::error;

pub fn load(source: &str, scene_index: usize) -> Option<(Root, Scene)> {
    let (doc, buffers, images) = match gltf::import(source) {
        Ok(tuple) => tuple,
        Err(err) => {
            error!("glTF import failed: {err}");
            if let gltf::Error::Io(_) = err {
                error!("Hint: Are the .bin file(s) referenced by the .gltf file available?")
            }
            return None;
        }
    };
    let imp = Document {
        doc,
        buffers,
        images,
    };

    if scene_index >= imp.doc.scenes().len() {
        error!(
            "Scene index too high - file has only {} scene(s)",
            imp.doc.scenes().len()
        );
        return None;
    }
    let base_path = Path::new(source);
    let mut root = Root::from_gltf(&imp, base_path);
    let scene = Scene::from_gltf(&imp.doc.scenes().nth(scene_index).unwrap(), &mut root);

    Some((root, scene))
}
