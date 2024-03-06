use std::{path::Path, rc::Rc};

use cgmath::{Vector3, Vector4};

use crate::gltf::{Document, GltfAlphaMode, GltfMaterial, GltfTexture, Root, Texture};

#[derive(Debug)]
pub struct Material {
    pub index: Option<usize>,
    pub name: Option<String>,

    // pbr_metallic_roughness properties
    pub base_color_factor: Vector4<f32>,
    pub base_color_texture: Option<Rc<Texture>>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture: Option<Rc<Texture>>,

    pub normal_texture: Option<Rc<Texture>>,
    pub normal_scale: Option<f32>,

    pub occlusion_texture: Option<Rc<Texture>>,
    pub occlusion_strength: f32,
    pub emissive_factor: Vector3<f32>,
    pub emissive_texture: Option<Rc<Texture>>,

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

        let mut material = Material {
            index: gltf_material.index(),
            name: gltf_material.name().map(|s| s.into()),
            base_color_factor: pbr.base_color_factor().into(),
            base_color_texture: None,
            metallic_factor: pbr.metallic_factor(),
            roughness_factor: pbr.roughness_factor(),
            metallic_roughness_texture: None,

            normal_texture: None,
            normal_scale: None,

            occlusion_texture: None,
            occlusion_strength: 0.0,

            emissive_factor: gltf_material.emissive_factor().into(),
            emissive_texture: None,

            alpha_cutoff: gltf_material.alpha_cutoff().unwrap_or_default(),
            alpha_mode: gltf_material.alpha_mode(),

            double_sided: gltf_material.double_sided(),
        };

        if let Some(color_info) = pbr.base_color_texture() {
            material.base_color_texture = Some(load_texture(
                &color_info.texture(),
                color_info.tex_coord(),
                root,
                document,
                base_path,
            ));
        }
        if let Some(mr_info) = pbr.metallic_roughness_texture() {
            material.metallic_roughness_texture = Some(load_texture(
                &mr_info.texture(),
                mr_info.tex_coord(),
                root,
                document,
                base_path,
            ));
        }
        if let Some(normal_texture) = gltf_material.normal_texture() {
            material.normal_texture = Some(load_texture(
                &normal_texture.texture(),
                normal_texture.tex_coord(),
                root,
                document,
                base_path,
            ));
            material.normal_scale = Some(normal_texture.scale());
        }
        if let Some(occ_texture) = gltf_material.occlusion_texture() {
            material.occlusion_texture = Some(load_texture(
                &occ_texture.texture(),
                occ_texture.tex_coord(),
                root,
                document,
                base_path,
            ));
            material.occlusion_strength = occ_texture.strength();
        }
        if let Some(em_info) = gltf_material.emissive_texture() {
            material.emissive_texture = Some(load_texture(
                &em_info.texture(),
                em_info.tex_coord(),
                root,
                document,
                base_path,
            ));
        }

        material
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
