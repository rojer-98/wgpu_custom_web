use std::{path::Path, rc::Rc};

use cgmath::{Vector3, Vector4};

use crate::gltf::{Document, GltfAlphaMode, GltfMaterial, GltfTexture, Root, Texture};

#[derive(Debug)]
pub struct BaseColorTexture {
    pub factor: Vector4<f32>,
    pub texture: Rc<Texture>,
}

#[derive(Debug)]
pub struct MRTexture {
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub texture: Rc<Texture>,
}

#[derive(Debug)]
pub struct NormalTexture {
    pub scale: f32,
    pub texture: Rc<Texture>,
}

#[derive(Debug)]
pub struct OcclusionTexture {
    pub texture: Rc<Texture>,
    pub strength: f32,
}

#[derive(Debug)]
pub struct EmissiveTexture {
    pub factor: Vector3<f32>,
    pub texture: Rc<Texture>,
}

#[derive(Debug)]
pub struct Material {
    pub index: Option<usize>,
    pub name: Option<String>,

    pub base_color: Option<BaseColorTexture>,
    pub mr: Option<MRTexture>,
    pub normal: Option<NormalTexture>,
    pub occlusion: Option<OcclusionTexture>,
    pub emissive: Option<EmissiveTexture>,

    pub alpha_cutoff: f32,
    pub alpha_mode: GltfAlphaMode,

    pub double_sided: bool,
}

impl Material {
    pub fn new(
        gltf_material: &GltfMaterial<'_>,
        root: &mut Root,
        document: &Document,
        base_path: &Path,
    ) -> Material {
        let pbr = gltf_material.pbr_metallic_roughness();

        let base_color = pbr.base_color_texture().map(|color_info| BaseColorTexture {
            texture: load_texture(
                &color_info.texture(),
                color_info.tex_coord(),
                root,
                document,
                base_path,
            ),
            factor: pbr.base_color_factor().into(),
        });
        let mr = pbr.metallic_roughness_texture().map(|mr_info| MRTexture {
            texture: load_texture(
                &mr_info.texture(),
                mr_info.tex_coord(),
                root,
                document,
                base_path,
            ),
            roughness_factor: pbr.roughness_factor(),
            metallic_factor: pbr.metallic_factor(),
        });
        let normal = gltf_material
            .normal_texture()
            .map(|normal_texture| NormalTexture {
                texture: load_texture(
                    &normal_texture.texture(),
                    normal_texture.tex_coord(),
                    root,
                    document,
                    base_path,
                ),
                scale: normal_texture.scale(),
            });

        let occlusion = gltf_material
            .occlusion_texture()
            .map(|occ_texture| OcclusionTexture {
                texture: load_texture(
                    &occ_texture.texture(),
                    occ_texture.tex_coord(),
                    root,
                    document,
                    base_path,
                ),
                strength: occ_texture.strength(),
            });

        let emissive = gltf_material
            .emissive_texture()
            .map(|em_info| EmissiveTexture {
                texture: load_texture(
                    &em_info.texture(),
                    em_info.tex_coord(),
                    root,
                    document,
                    base_path,
                ),
                factor: gltf_material.emissive_factor().into(),
            });

        Material {
            index: gltf_material.index(),
            name: gltf_material.name().map(|s| s.into()),

            emissive,
            occlusion,
            base_color,
            mr,
            normal,

            alpha_cutoff: gltf_material.alpha_cutoff().unwrap_or_default(),
            alpha_mode: gltf_material.alpha_mode(),

            double_sided: gltf_material.double_sided(),
        }
    }
}

fn load_texture(
    g_texture: &GltfTexture<'_>,
    tex_coord: u32,
    root: &mut Root,
    document: &Document,
    base_path: &Path,
) -> Rc<Texture> {
    if let Some(tex) = root
        .textures
        .iter()
        .find(|tex| (***tex).index == g_texture.index())
    {
        return Rc::clone(tex);
    }

    let texture = Rc::new(Texture::new(g_texture, tex_coord, document, base_path));
    root.textures.push(Rc::clone(&texture));

    texture
}
