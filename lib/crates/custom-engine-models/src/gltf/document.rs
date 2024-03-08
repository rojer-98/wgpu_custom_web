use crate::gltf::{GltfBufferData, GltfDocument, GltfImageData};

#[derive(Debug)]
pub struct Document {
    pub inner: GltfDocument,
    pub buffers: Vec<GltfBufferData>,
    pub images: Vec<GltfImageData>,
}
