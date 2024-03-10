#[derive(Debug)]
pub struct Document {
    pub inner: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}
