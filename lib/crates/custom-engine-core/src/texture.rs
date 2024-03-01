mod depth;
mod hdr;
mod render;

pub use depth::*;
pub use hdr::*;
pub use render::*;

use derive_more::Constructor;

use crate::buffer::Buffer;

#[derive(Debug)]
pub enum TextureKind {
    Render,
    NormalMap,
    Depth,
    HDR,
}

impl From<TextureKind> for wgpu::TextureFormat {
    fn from(value: TextureKind) -> Self {
        use TextureKind::*;

        match value {
            Render => wgpu::TextureFormat::Rgba8UnormSrgb,
            NormalMap => wgpu::TextureFormat::Rgba8Unorm,
            Depth => wgpu::TextureFormat::Depth32Float,
            HDR => wgpu::TextureFormat::Rgba16Float,
        }
    }
}

#[derive(Debug, Constructor)]
pub struct CopyTextureParams<'a> {
    pub buffer: &'a Buffer,
    pub texture: &'a RenderTexture,
}
