use std::{
    collections::HashMap,
    io::{BufReader, Cursor},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use log::{error, info};
use tobj::{LoadOptions, Material, Model};

use crate::utils::get_data;

#[derive(Debug, Default)]
pub struct FileTextures {
    pub ambient_texture: Option<Vec<u8>>,
    pub diffuse_texture: Option<Vec<u8>>,
    pub specular_texture: Option<Vec<u8>>,
    pub shininess_texture: Option<Vec<u8>>,
    pub normal_texture: Option<Vec<u8>>,
    pub dissolve_texture: Option<Vec<u8>>,
}

impl FileTextures {
    pub fn new(current_path: &PathBuf, m: &Material) -> Self {
        FileTextures {
            dissolve_texture: m
                .dissolve_texture
                .clone()
                .map(|t| {
                    let mut current_path = current_path.clone();
                    current_path.push(t);
                    let current_path = current_path.to_str().unwrap();

                    get_data(current_path)
                })
                .flatten(),
            normal_texture: m
                .normal_texture
                .clone()
                .map(|t| {
                    let mut current_path = current_path.clone();
                    current_path.push(t);
                    let current_path = current_path.to_str().unwrap();

                    get_data(current_path)
                })
                .flatten(),
            shininess_texture: m
                .shininess_texture
                .clone()
                .map(|t| {
                    let mut current_path = current_path.clone();
                    current_path.push(t);
                    let current_path = current_path.to_str().unwrap();

                    get_data(current_path)
                })
                .flatten(),
            specular_texture: m
                .specular_texture
                .clone()
                .map(|t| {
                    let mut current_path = current_path.clone();
                    current_path.push(t);
                    let current_path = current_path.to_str().unwrap();

                    get_data(current_path)
                })
                .flatten(),
            diffuse_texture: m
                .diffuse_texture
                .clone()
                .map(|t| {
                    let mut current_path = current_path.clone();
                    current_path.push(t);
                    let current_path = current_path.to_str().unwrap();

                    get_data(current_path)
                })
                .flatten(),
            ambient_texture: m
                .ambient_texture
                .clone()
                .map(|t| {
                    let mut current_path = current_path.clone();
                    current_path.push(t);
                    let current_path = current_path.to_str().unwrap();

                    get_data(current_path)
                })
                .flatten(),
        }
    }
}

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct LoadedMaterial {
    pub material: Material,
    #[derivative(Debug = "ignore")]
    pub files: FileTextures,
}

#[derive(Debug, Default)]
pub struct ObjFile {
    pub name: String,
    pub materials: HashMap<usize, LoadedMaterial>,
    pub models: HashMap<usize, Model>,
}

impl ObjFile {
    pub fn new_data(
        name: String,
        obj_data: Vec<u8>,
        mtl_obj_data: HashMap<String, Vec<u8>>,
        mut textures: HashMap<String, FileTextures>,
        load_options: LoadOptions,
    ) -> Result<Self> {
        let mut obj_reader = BufReader::new(Cursor::new(obj_data));
        let (models, materials) = {
            let (mdls, mat_res) = tobj::load_obj_buf(&mut obj_reader, &load_options, |p| {
                if let Some(p) = p.file_name() {
                    // Be sure `path` is correct
                    let f = p.to_str().ok_or(tobj::LoadError::MaterialParseError)?;
                    let mtl_data = mtl_obj_data
                        .get(f)
                        .ok_or(tobj::LoadError::MaterialParseError)?;

                    return tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mtl_data)));
                }

                tobj::MTLLoadResult::Err(tobj::LoadError::MaterialParseError)
            })?;
            if let Err(e) = mat_res {
                error!("{e}")
            }

            (mdls, mat_res?)
        };

        let models = models.into_iter().enumerate().collect::<HashMap<_, _>>();

        let materials = materials
            .into_iter()
            .enumerate()
            .map(|(i, m)| -> Result<(usize, LoadedMaterial)> {
                Ok((
                    i,
                    LoadedMaterial {
                        files: textures
                            .remove(&m.name)
                            .ok_or(anyhow!("cannot find loaded textures in `{}`.mtl", m.name))?,
                        material: m,
                    },
                ))
            })
            .filter_map(|res| {
                if let Err(e) = res {
                    error!("{e}");
                    None
                } else {
                    Some(res.unwrap())
                }
            })
            .collect::<HashMap<_, _>>();

        Ok(Self {
            models,
            materials,
            name,
        })
    }

    pub fn new(file_name: &str) -> Result<Self> {
        let (models, materials) = {
            let (mdls, mat_res) = tobj::load_obj(
                file_name,
                &LoadOptions {
                    single_index: true,
                    triangulate: true,
                    ..Default::default()
                },
            )?;
            if let Err(e) = mat_res {
                error!("{e}")
            }

            (mdls, mat_res?)
        };

        let models = models.into_iter().enumerate().collect::<HashMap<_, _>>();
        let mut current_path = PathBuf::from(file_name);
        current_path.pop();

        let materials = materials
            .into_iter()
            .enumerate()
            .map(|(i, m)| {
                (
                    i,
                    LoadedMaterial {
                        files: FileTextures::new(&current_path, &m),
                        material: m,
                    },
                )
            })
            .collect::<HashMap<_, _>>();
        let name = file_name.to_string();

        Ok(Self {
            models,
            materials,
            name,
        })
    }

    pub fn info(&self) {
        let Self {
            materials, models, ..
        } = self;

        info!("Number of models          = {}", models.len());
        info!("Number of materials       = {}", materials.len());

        for (i, m) in models.iter() {
            let mesh = &m.mesh;
            info!("");
            info!("model[{}].name             = \'{}\'", i, m.name);
            info!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

            info!(
                "model[{}].face_count       = {}",
                i,
                mesh.face_arities.len()
            );

            let mut next_face = 0;
            for face in 0..mesh.face_arities.len() {
                let end = next_face + mesh.face_arities[face] as usize;

                let face_indices = &mesh.indices[next_face..end];
                info!(" face[{}].indices          = {:?}", face, face_indices);

                if !mesh.texcoord_indices.is_empty() {
                    let texcoord_face_indices = &mesh.texcoord_indices[next_face..end];
                    info!(
                        " face[{}].texcoord_indices = {:?}",
                        face, texcoord_face_indices
                    );
                }
                if !mesh.normal_indices.is_empty() {
                    let normal_face_indices = &mesh.normal_indices[next_face..end];
                    info!(
                        " face[{}].normal_indices   = {:?}",
                        face, normal_face_indices
                    );
                }

                next_face = end;
            }

            info!(
                "model[{}].positions        = {}",
                i,
                mesh.positions.len() / 3
            );
            assert!(mesh.positions.len() % 3 == 0);

            for vtx in 0..mesh.positions.len() / 3 {
                info!(
                    "              position[{}] = ({}, {}, {})",
                    vtx,
                    mesh.positions[3 * vtx],
                    mesh.positions[3 * vtx + 1],
                    mesh.positions[3 * vtx + 2]
                );
            }
        }

        for (i, lm) in materials.iter() {
            let LoadedMaterial { material: m, .. } = lm;

            info!("material[{}].name = \'{}\'", i, m.name);
            if let Some(a) = m.ambient.as_ref() {
                info!("    material.Ka = ({}, {}, {})", a[0], a[1], a[2]);
            }
            if let Some(d) = m.diffuse.as_ref() {
                info!("    material.Kd = ({:?}, {:?}, {:?})", d[0], d[1], d[2]);
            }
            if let Some(s) = m.specular {
                info!("    material.Ks = ({:?}, {:?}, {:?})", s[0], s[1], s[2]);
            }
            info!("    material.Ns = {:?}", m.shininess);
            info!("    material.d = {:?}", m.dissolve);
            info!("    material.map_Ka = {:?}", m.ambient_texture);
            info!("    material.map_Kd = {:?}", m.diffuse_texture);
            info!("    material.map_Ks = {:?}", m.specular_texture);
            info!("    material.map_Ns = {:?}", m.shininess_texture);
            info!("    material.map_Bump = {:?}", m.normal_texture);
            info!("    material.map_d = {:?}", m.dissolve_texture);

            for (k, v) in &m.unknown_param {
                info!("    material.{} = {}", k, v);
            }
        }
    }
}
