mod depth;
mod render;

pub use depth::*;
pub use render::*;

use derive_more::Constructor;

use crate::buffer::Buffer;

#[derive(Debug)]
pub enum TextureKind {
    Surface,
    Render,
    NormalMap,
    Depth,
    HDR,
}

impl From<TextureKind> for wgpu::TextureFormat {
    fn from(value: TextureKind) -> Self {
        use TextureKind::*;

        match value {
            Surface => {
                if cfg!(target_arch = "wasm32") {
                    wgpu::TextureFormat::Rgba8UnormSrgb
                } else {
                    wgpu::TextureFormat::Bgra8UnormSrgb
                }
            }
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

impl<'a> CopyTextureParams<'a> {
    pub fn process(&self, encoder: &mut wgpu::CommandEncoder) {
        let CopyTextureParams { buffer, texture } = self;

        texture.load_to_buffer(encoder, buffer);
    }
}
