use crate::gltf::{GltfBufferData, GltfDocument, GltfImageData};

pub struct Document {
    pub doc: GltfDocument,
    pub buffers: Vec<GltfBufferData>,
    pub images: Vec<GltfImageData>,
}
