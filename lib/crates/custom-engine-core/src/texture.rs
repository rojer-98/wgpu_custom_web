mod depth;
mod render;

pub use depth::*;
pub use render::*;

use crate::buffer::Buffer;

#[derive(Debug)]
pub struct CopyTextureParams<'a> {
    pub buffer: &'a Buffer,
    pub texture: &'a RenderTexture,
}

impl<'a> CopyTextureParams<'a> {
    pub fn new(buffer: &'a Buffer, texture: &'a RenderTexture) -> Self {
        Self { texture, buffer }
    }
}
