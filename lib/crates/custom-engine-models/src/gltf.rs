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

use anyhow::{anyhow, Result};
use gltf::{
    buffer::Data as GltfBufferData, camera::Camera as GltfCamera, image::Data as GltfImageData,
    material::AlphaMode as GltfAlphaMode, material::Material as GltfMaterial,
    mesh::Mesh as GltfMesh, mesh::Mode as GltfMode, scene::Scene as GltfScene,
    texture::Texture as GltfTexture, Document as GltfDocument, Primitive as GltfPrimitive,
};

use crate::utils::get_data;

#[derive(Debug)]
pub struct GltfFile {
    pub name: String,

    pub doc: Document,
    pub root: Root,
}

impl GltfFile {
    pub async fn new(file_name: &str) -> Result<Self> {
        let (inner, buffers, images) = if cfg!(target_arch = "wasm32") {
            let slice = get_data(file_name)
                .await
                .ok_or(anyhow!("File source of `{file_name}` is not availiable"))?;
            gltf::import_slice(slice)?
        } else {
            gltf::import(file_name)?
        };

        let doc = Document {
            inner,
            buffers,
            images,
        };
        let base_path = Path::new(file_name);
        let name = base_path
            .file_name()
            .ok_or(anyhow!("File name is not available"))?
            .to_str()
            .unwrap()
            .to_string();
        let root = Root::new(&doc, base_path);

        Ok(Self { name, root, doc })
    }

    pub fn scene(&mut self, scene_index: usize) -> Result<Scene> {
        let mut scenes = self.doc.inner.scenes();
        let scenes_len = scenes.len();

        if scene_index >= scenes_len {
            return Err(anyhow!(
                "Scene index too high - file has only {scenes_len} scene(s)",
            ));
        }

        let scene = scenes.nth(scene_index).unwrap();

        Ok(Scene::new(&scene, &mut self.root))
    }

    pub fn scenes(&mut self) -> Result<Vec<Scene>> {
        let mut scenes = self.doc.inner.scenes();
        let scenes_len = scenes.len();

        Ok((0..scenes_len)
            .into_iter()
            .map(|scene_index| Scene::new(&scenes.nth(scene_index).unwrap(), &mut self.root))
            .collect::<Vec<_>>())
    }
}
