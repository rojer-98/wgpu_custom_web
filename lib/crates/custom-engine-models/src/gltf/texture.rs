use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use anyhow::Result;
use base64::prelude::*;
use gltf::{image::Source, texture::Texture as GltfTexture};

use crate::gltf::document::Document;

#[derive(Debug)]
pub struct Texture {
    pub index: usize, // glTF index
    pub name: Option<String>,

    pub tex_coord: u32, // the tex coord set to use
    pub dyn_image: Vec<u8>,
}

impl Texture {
    pub fn new(
        g_texture: &GltfTexture<'_>,
        tex_coord: u32,
        document: &Document,
        base_path: &Path,
    ) -> Result<Texture> {
        let buffers = &document.buffers;

        let g_img = g_texture.source();
        let dyn_image = match g_img.source() {
            Source::View { view, .. } => {
                let parent_buffer_data = &buffers[view.buffer().index()].0;
                let begin = view.offset();
                let end = begin + view.length();
                let data = &parent_buffer_data[begin..end];

                data.to_vec()
            }
            Source::Uri { uri, .. } => {
                if uri.starts_with("data:") {
                    let encoded = uri.split(',').nth(1).unwrap();
                    let data = BASE64_STANDARD.decode(&encoded).unwrap();

                    data
                } else {
                    let path = base_path
                        .parent()
                        .unwrap_or_else(|| Path::new("./"))
                        .join(uri);
                    let file = File::open(path).unwrap();
                    let mut reader = BufReader::new(file);
                    let mut data = vec![];
                    reader.read_to_end(&mut data)?;

                    data
                }
            }
        };

        Ok(Texture {
            index: g_texture.index(),
            name: g_texture.name().map(|s| s.into()),
            tex_coord,
            dyn_image,
        })
    }
}
