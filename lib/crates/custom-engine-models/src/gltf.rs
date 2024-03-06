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

use gltf::{
    buffer::Data as GltfBufferData, camera::Camera as GltfCamera, image::Data as GltfImageData,
    material::Material as GltfMaterial, mesh::Mesh as GltfMesh, scene::Scene as GltfScene,
    texture::Texture as GltfTexture, Document as GltfDocument, Primitive as GltfPrimitive,
};
