use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Document {
    pub inner: gltf::Document,
    #[derivative(Debug = "ignore")]
    pub buffers: Vec<gltf::buffer::Data>,
    #[derivative(Debug = "ignore")]
    pub images: Vec<gltf::image::Data>,
}
