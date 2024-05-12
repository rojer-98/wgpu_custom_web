use std::{path::Path, rc::Rc};

use cgmath::{Vector3, Vector4};

use crate::gltf::{Document, Root, Texture};

#[derive(Debug, Clone)]
pub struct BaseColorTexture {
    pub factor: Vector4<f32>,
    pub texture: Rc<Texture>,
}

#[derive(Debug, Clone)]
pub struct MRTexture {
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub texture: Rc<Texture>,
}

#[derive(Debug, Clone)]
pub struct NormalTexture {
    pub scale: f32,
    pub texture: Rc<Texture>,
}

#[derive(Debug, Clone)]
pub struct OcclusionTexture {
    pub texture: Rc<Texture>,
    pub strength: f32,
}

#[derive(Debug, Clone)]
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
    pub alpha_mode: gltf::material::AlphaMode,

    pub double_sided: bool,
}

impl Material {
    pub fn new(
        gltf_material: &gltf::Material<'_>,
        root: &mut Root,
        document: &Document,
        base_path: &Path,
    ) -> Material {
        let pbr = gltf_material.pbr_metallic_roughness();

        let base_color = if let Some(color_info) = pbr.base_color_texture() {
            Some(BaseColorTexture {
                texture: load_texture(
                    &color_info.texture(),
                    color_info.tex_coord(),
                    root,
                    document,
                    base_path,
                ),
                factor: pbr.base_color_factor().into(),
            })
        } else {
            None
        };
        let mr = if let Some(mr_info) = pbr.metallic_roughness_texture() {
            Some(MRTexture {
                texture: load_texture(
                    &mr_info.texture(),
                    mr_info.tex_coord(),
                    root,
                    document,
                    base_path,
                ),
                roughness_factor: pbr.roughness_factor(),
                metallic_factor: pbr.metallic_factor(),
            })
        } else {
            None
        };
        let normal = if let Some(normal_texture) = gltf_material.normal_texture() {
            Some(NormalTexture {
                texture: load_texture(
                    &normal_texture.texture(),
                    normal_texture.tex_coord(),
                    root,
                    document,
                    base_path,
                ),
                scale: normal_texture.scale(),
            })
        } else {
            None
        };

        let occlusion = if let Some(occ_texture) = gltf_material.occlusion_texture() {
            Some(OcclusionTexture {
                texture: load_texture(
                    &occ_texture.texture(),
                    occ_texture.tex_coord(),
                    root,
                    document,
                    base_path,
                ),
                strength: occ_texture.strength(),
            })
        } else {
            None
        };

        let emissive = if let Some(em_info) = gltf_material.emissive_texture() {
            Some(EmissiveTexture {
                texture: load_texture(
                    &em_info.texture(),
                    em_info.tex_coord(),
                    root,
                    document,
                    base_path,
                ),
                factor: gltf_material.emissive_factor().into(),
            })
        } else {
            None
        };

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
    g_texture: &gltf::Texture<'_>,
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

    let texture = match Texture::new(g_texture, tex_coord, document, base_path) {
        Ok(t) => t,
        Err(e) => panic!("Load texture: {e}"),
    };
    let texture = Rc::new(texture);

    root.textures.push(Rc::clone(&texture));

    texture
}
