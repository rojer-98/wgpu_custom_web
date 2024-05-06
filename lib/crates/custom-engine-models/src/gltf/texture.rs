use std::path::Path;

use anyhow::{anyhow, Result};
use base64::prelude::*;
use derivative::Derivative;

use custom_engine_utils::get_data;

use crate::gltf::document::Document;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Texture {
    pub index: usize, // glTF index
    pub name: Option<String>,

    pub tex_coord: u32, // the tex coord set to use
    #[derivative(Debug = "ignore")]
    pub dyn_image: Vec<u8>,
}

impl Texture {
    pub async fn new(
        g_texture: &gltf::Texture<'_>,
        tex_coord: u32,
        document: &Document,
        base_path: &Path,
    ) -> Result<Texture> {
        use gltf::image::Source;

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
                    get_data(
                        base_path
                            .parent()
                            .unwrap_or_else(|| Path::new("./"))
                            .join(uri)
                            .to_str()
                            .ok_or(anyhow!("Base path is wrong"))?,
                    )
                    .await
                    .ok_or(anyhow!("Source URI `{uri}` data is not found"))?
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
