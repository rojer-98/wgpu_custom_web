use std::{borrow::Cow, str::from_utf8};

use derive_more::Display;
use rust_embed::RustEmbed;

use crate::errors::EngineError;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/assets/shaders"]
#[include = "*.wgsl"]
pub struct ShaderFiles;

#[derive(Debug, Display)]
pub enum ShaderKind {
    #[display(fmt = "light")]
    Light,
    #[display(fmt = "model")]
    Model,
    #[display(fmt = "simple")]
    Simple,
    #[display(fmt = "custom")]
    Custom,
    #[display(fmt = "texture")]
    Texture,
    #[display(fmt = "hdr")]
    HDR,
}

impl ShaderFiles {
    pub fn get_file_data(kind: ShaderKind) -> Result<wgpu::ShaderSource<'static>, EngineError> {
        let sh_name = format!("{kind}.wgsl");
        let sh_file = ShaderFiles::get(&sh_name).ok_or(EngineError::FileNotFound(sh_name))?;
        let sh_data = from_utf8(&sh_file.data)?.to_string();

        Ok(wgpu::ShaderSource::Wgsl(Cow::Owned(sh_data)))
    }
}

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/assets/spv"]
#[include = "*.spv"]
pub struct SpvFiles;

impl SpvFiles {
    pub fn get_file_data(kind: ShaderKind) -> Result<Vec<u32>, EngineError> {
        let spv_name = format!("{kind}.spv");
        let spv_file = SpvFiles::get(&spv_name).ok_or(EngineError::FileNotFound(spv_name))?;

        Ok(spv_file
            .data
            .to_vec()
            .chunks(4)
            .map(|bytes| {
                (bytes[0] as u32)
                    | ((bytes[1] as u32) << 8)
                    | ((bytes[2] as u32) << 16)
                    | ((bytes[3] as u32) << 24)
            })
            .collect())
    }
}
