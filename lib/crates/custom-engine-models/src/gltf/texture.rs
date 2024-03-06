use std::{fs::File, io::BufReader, path::Path};

use base64::prelude::*;
use gltf::{image::Source, texture::Texture as GltfTexture};
use image::{DynamicImage, ImageFormat};

use crate::gltf::document::Document;

#[derive(Debug)]
pub struct Texture {
    pub index: usize, // glTF index
    pub name: Option<String>,

    pub tex_coord: u32, // the tex coord set to use
    pub dyn_image: DynamicImage,
}

impl Texture {
    pub fn new(
        g_texture: &GltfTexture<'_>,
        tex_coord: u32,
        document: &Document,
        base_path: &Path,
    ) -> Texture {
        let buffers = &document.buffers;

        let g_img = g_texture.source();
        let img = match g_img.source() {
            Source::View { view, mime_type } => {
                let parent_buffer_data = &buffers[view.buffer().index()].0;
                let begin = view.offset();
                let end = begin + view.length();
                let data = &parent_buffer_data[begin..end];
                match mime_type {
                    "image/jpeg" => image::load_from_memory_with_format(data, ImageFormat::Jpeg),
                    "image/png" => image::load_from_memory_with_format(data, ImageFormat::Png),
                    _ => panic!(
                        "unsupported image type (image: {}, mime_type: {})",
                        g_img.index(),
                        mime_type
                    ),
                }
            }
            Source::Uri { uri, mime_type } => {
                if uri.starts_with("data:") {
                    let encoded = uri.split(',').nth(1).unwrap();
                    let data = BASE64_STANDARD.decode(&encoded).unwrap();
                    let mime_type = if let Some(ty) = mime_type {
                        ty
                    } else {
                        uri.split(',')
                            .nth(0)
                            .unwrap()
                            .split(':')
                            .nth(1)
                            .unwrap()
                            .split(';')
                            .nth(0)
                            .unwrap()
                    };

                    match mime_type {
                        "image/jpeg" => {
                            image::load_from_memory_with_format(&data, ImageFormat::Jpeg)
                        }
                        "image/png" => image::load_from_memory_with_format(&data, ImageFormat::Png),
                        _ => panic!(
                            "unsupported image type (image: {}, mime_type: {})",
                            g_img.index(),
                            mime_type
                        ),
                    }
                } else if let Some(mime_type) = mime_type {
                    let path = base_path
                        .parent()
                        .unwrap_or_else(|| Path::new("./"))
                        .join(uri);
                    let file = File::open(path).unwrap();
                    let reader = BufReader::new(file);
                    match mime_type {
                        "image/jpeg" => image::load(reader, ImageFormat::Jpeg),
                        "image/png" => image::load(reader, ImageFormat::Png),
                        _ => panic!(
                            "unsupported image type (image: {}, mime_type: {})",
                            g_img.index(),
                            mime_type
                        ),
                    }
                } else {
                    let path = base_path
                        .parent()
                        .unwrap_or_else(|| Path::new("./"))
                        .join(uri);
                    image::open(path)
                }
            }
        };

        // TODO: handle I/O problems
        let dyn_image = img.expect("Image loading failed.");

        Texture {
            index: g_texture.index(),
            name: g_texture.name().map(|s| s.into()),
            tex_coord,
            dyn_image,
        }
    }
}
