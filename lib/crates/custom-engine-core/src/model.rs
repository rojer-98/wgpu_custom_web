pub mod material;
pub mod mesh;

use custom_engine_derive::VertexLayout;
use custom_engine_models::{gltf::GltfFile, obj::ObjFile};

use cgmath::{Vector2, Vector3};
use log::{debug, error};

use crate::{
    bind_group::layout::{BindGroupLayout, BindGroupLayoutBuilder},
    errors::CoreError,
    model::{
        material::{Material, MaterialBuilder, MaterialTextureParams},
        mesh::{Mesh, MeshBuilder},
    },
    traits::{Builder, VertexLayout},
};

#[derive(Debug)]
pub struct TextureParams {
    pub view_binding: u32,
    pub sampler_binding: u32,
    pub format: wgpu::TextureFormat,
}

impl TextureParams {
    pub fn process<'a>(
        &'a self,
        bind_group_layout: BindGroupLayoutBuilder<'a>,
    ) -> BindGroupLayoutBuilder<'_> {
        bind_group_layout
            .entries(wgpu::BindGroupLayoutEntry {
                binding: self.view_binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            })
            .entries(wgpu::BindGroupLayoutEntry {
                binding: self.sampler_binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            })
    }
}

#[derive(Debug)]
pub enum ModelFile {
    Obj(ObjFile),
    Gltf((usize, GltfFile)),
}

impl From<ObjFile> for ModelFile {
    fn from(value: ObjFile) -> Self {
        Self::Obj(value)
    }
}

impl From<GltfFile> for ModelFile {
    fn from(value: GltfFile) -> Self {
        Self::Gltf((0, value))
    }
}

impl From<(usize, GltfFile)> for ModelFile {
    fn from(value: (usize, GltfFile)) -> Self {
        Self::Gltf((value.0, value.1))
    }
}

#[derive(Debug)]
pub struct Model {
    pub id: usize,

    bind_group_layout: BindGroupLayout,
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
}

impl Model {
    #[inline]
    pub fn load(&self, queue: &wgpu::Queue) {
        self.materials
            .iter()
            .for_each(|m| m.store_textures_to_memory(queue));
    }

    #[inline]
    pub fn get_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        ModelRaw::desc()
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn meshes(&self) -> &[Mesh] {
        &self.meshes
    }

    pub fn materials(&self) -> &[Material] {
        &self.materials
    }
}

#[derive(Debug)]
pub struct ModelBuilder<'a> {
    id: Option<usize>,
    file: Option<ModelFile>,

    mesh_vertex_binding: Option<u32>,

    diffuse: Option<TextureParams>,
    normal: Option<TextureParams>,
    mr: Option<TextureParams>,
    emissive: Option<TextureParams>,
    occlusion: Option<TextureParams>,

    device: &'a wgpu::Device,
}

impl<'a> Builder<'a> for ModelBuilder<'a> {
    type Final = Model;

    fn new(device: &'a wgpu::Device) -> Self
    where
        Self: Sized,
    {
        Self {
            id: None,
            file: None,

            mesh_vertex_binding: None,

            diffuse: None,
            normal: None,
            mr: None,
            emissive: None,
            occlusion: None,

            device,
        }
    }

    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            id: Some(id),
            file: None,

            mesh_vertex_binding: None,

            diffuse: None,
            normal: None,
            mr: None,
            emissive: None,
            occlusion: None,

            device,
        }
    }

    fn build(self) -> Result<Self::Final, CoreError>
    where
        Self: Sized,
    {
        use ModelFile::*;

        let id = self.id.unwrap_or_default();
        let model_name = format!("Model: {id}");

        let diffuse = self
            .diffuse
            .ok_or(CoreError::EmptyDiffuseTexture(model_name.clone()))?;
        let mesh_vertex_binding = self.mesh_vertex_binding.unwrap_or_default();

        let file = self
            .file
            .ok_or(CoreError::EmptyModelFile(model_name.to_string()))?;

        let bgl_name = format!("Bind Group Layout of `{model_name}`");
        let mut bind_group_layout =
            diffuse.process(BindGroupLayoutBuilder::new(self.device).label(&bgl_name));

        if let Some(tp) = self.normal.as_ref() {
            bind_group_layout = tp.process(bind_group_layout)
        }
        if let Some(tp) = self.mr.as_ref() {
            bind_group_layout = tp.process(bind_group_layout)
        }
        if let Some(tp) = self.emissive.as_ref() {
            bind_group_layout = tp.process(bind_group_layout)
        }
        if let Some(tp) = self.occlusion.as_ref() {
            bind_group_layout = tp.process(bind_group_layout)
        }

        let bind_group_layout = bind_group_layout.build()?;

        match file {
            Obj(obj_file) => {
                let materials = obj_file
                    .materials
                    .iter()
                    .map(|(i, lm)| -> Result<Material, CoreError> {
                        let mut mb = MaterialBuilder::new(self.device).layout(&bind_group_layout);
                        let texture_name = lm.material.name.to_string();
                        debug!(
                            "
Proceed material: `{texture_name}:{i}`:
            "
                        );

                        let diffuse_texture_data =
                            &lm.files
                                .diffuse_texture
                                .clone()
                                .ok_or(CoreError::EmptyData(format!(
                                    "Diffuse texture: {:?}",
                                    lm.material.diffuse_texture
                                )))?;

                        let diffuse = MaterialTextureParams {
                            format: diffuse.format,
                            texture_data: Some(diffuse_texture_data),
                            view_binding: diffuse.view_binding,
                            sampler_binding: diffuse.sampler_binding,
                        };

                        mb = mb.diffuse(diffuse);

                        if let Some(normal) = self.normal.as_ref() {
                            let normal_texture_data =
                                lm.files.normal_texture.as_ref().map(|d| d.as_slice());
                            let normal = MaterialTextureParams {
                                format: normal.format,
                                texture_data: normal_texture_data,
                                view_binding: normal.view_binding,
                                sampler_binding: normal.sampler_binding,
                            };

                            mb = mb.normal(normal);
                        }

                        Ok(mb.build()?)
                    })
                    .filter_map(|m_res| {
                        if let Err(e) = m_res {
                            error!("{e}");
                            None
                        } else {
                            m_res.ok()
                        }
                    })
                    .collect::<Vec<_>>();

                let meshes = obj_file
                    .models
                    .into_values()
                    .map(|m| -> Result<Mesh, CoreError> {
                        let mut vertices = (0..m.mesh.positions.len() / 3)
                            .map(|i| ModelRaw {
                                position: [
                                    m.mesh.positions[i * 3],
                                    m.mesh.positions[i * 3 + 1],
                                    m.mesh.positions[i * 3 + 2],
                                ],
                                tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]]
                                    .into(),
                                normal: [
                                    m.mesh.normals[i * 3],
                                    m.mesh.normals[i * 3 + 1],
                                    m.mesh.normals[i * 3 + 2],
                                ],
                                tangent: [0.0; 3],
                                bitangent: [0.0; 3],
                            })
                            .collect::<Vec<_>>();

                        let indices = &m.mesh.indices;
                        let mut triangles_included = vec![0; vertices.len()];

                        for c in indices.chunks(3) {
                            let v0 = vertices[c[0] as usize];
                            let v1 = vertices[c[1] as usize];
                            let v2 = vertices[c[2] as usize];

                            let pos0: Vector3<_> = v0.position.into();
                            let pos1: Vector3<_> = v1.position.into();
                            let pos2: Vector3<_> = v2.position.into();

                            let uv0: Vector2<_> = v0.tex_coords.into();
                            let uv1: Vector2<_> = v1.tex_coords.into();
                            let uv2: Vector2<_> = v2.tex_coords.into();

                            let delta_pos1 = pos1 - pos0;
                            let delta_pos2 = pos2 - pos0;

                            let delta_uv1 = uv1 - uv0;
                            let delta_uv2 = uv2 - uv0;

                            let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);

                            let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                            let bitangent =
                                (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                            vertices[c[0] as usize].tangent =
                                (tangent + Vector3::from(vertices[c[0] as usize].tangent)).into();
                            vertices[c[1] as usize].tangent =
                                (tangent + Vector3::from(vertices[c[1] as usize].tangent)).into();
                            vertices[c[2] as usize].tangent =
                                (tangent + Vector3::from(vertices[c[2] as usize].tangent)).into();
                            vertices[c[0] as usize].bitangent = (bitangent
                                + Vector3::from(vertices[c[0] as usize].bitangent))
                            .into();
                            vertices[c[1] as usize].bitangent = (bitangent
                                + Vector3::from(vertices[c[1] as usize].bitangent))
                            .into();
                            vertices[c[2] as usize].bitangent = (bitangent
                                + Vector3::from(vertices[c[2] as usize].bitangent))
                            .into();

                            triangles_included[c[0] as usize] += 1;
                            triangles_included[c[1] as usize] += 1;
                            triangles_included[c[2] as usize] += 1;
                        }

                        for (i, n) in triangles_included.into_iter().enumerate() {
                            let denom = 1.0 / n as f32;
                            let v = &mut vertices[i];

                            v.tangent = (Vector3::from(v.tangent) * denom).into();
                            v.bitangent = (Vector3::from(v.bitangent) * denom).into();
                        }

                        let mesh = MeshBuilder::new(self.device)
                            .name(&obj_file.name)
                            .num_elements(m.mesh.indices.len() as u32)
                            .material(m.mesh.material_id.unwrap_or_default())
                            .vertex_buffer_data(&vertices)
                            .index_buffer_data(&m.mesh.indices)
                            .vertex_buffer_binding(mesh_vertex_binding)
                            .build()?;

                        Ok(mesh)
                    })
                    .filter_map(|m_res| {
                        if let Err(e) = m_res {
                            error!("{e}");
                            None
                        } else {
                            m_res.ok()
                        }
                    })
                    .collect::<Vec<_>>();

                Ok(Model {
                    id,
                    meshes,
                    materials,
                    bind_group_layout,
                })
            }
            Gltf((scene_id, mut gltf_file)) => {
                let scene = gltf_file.scene(scene_id)?;

                let mut f_m = vec![];
                let mut f_ms = vec![];

                for n_id in scene.nodes.iter() {
                    if let Some(node) = gltf_file.root.nodes.get(*n_id) {
                        if let Some(mesh) = node.mesh.as_ref() {
                            let materials = mesh
                                .primitives
                                .iter()
                                .map(|p| p.material.clone())
                                .enumerate()
                                .collect::<Vec<_>>();
                            let meshes = mesh.primitives.iter().enumerate().collect::<Vec<_>>();

                            for (i, m) in materials {
                                let mut mb =
                                    MaterialBuilder::new(self.device).layout(&bind_group_layout);
                                let texture_name = m.name.clone().unwrap();
                                debug!(
                                    "
Proceed material: `{texture_name}:{i}`:
            "
                                );

                                if let Some(base_color) = m.base_color.as_ref() {
                                    let diffuse_texture_data = &base_color.texture.dyn_image;
                                    let diffuse = MaterialTextureParams {
                                        format: diffuse.format,
                                        texture_data: Some(&diffuse_texture_data),
                                        view_binding: diffuse.view_binding,
                                        sampler_binding: diffuse.sampler_binding,
                                    };
                                    mb = mb.diffuse(diffuse);

                                    if let Some(normal) = self.normal.as_ref() {
                                        let normal_texture_data =
                                            &m.normal.as_ref().unwrap().texture.dyn_image;
                                        let normal = MaterialTextureParams {
                                            format: normal.format,
                                            texture_data: Some(normal_texture_data),
                                            view_binding: normal.view_binding,
                                            sampler_binding: normal.sampler_binding,
                                        };

                                        mb = mb.normal(normal);
                                    }
                                    /*
                                          let emissive_texture_data =
                                              m.emissive.as_ref().map(|d| d.texture.dyn_image.clone());
                                          let mr_texture_data =
                                              m.mr.as_ref().map(|d| d.texture.dyn_image.clone());
                                          let occlusion_texture_data =
                                              m.occlusion.as_ref().map(|d| d.texture.dyn_image.clone());

                                          let material = MaterialBuilder::new(self.device)
                                              .diffuse(diffuse)
                                              .normal(normal)
                                              .layout(&bind_group_layout)
                                              .build()
                                              .unwrap();

                                    */
                                    f_m.push(mb.build()?);
                                }
                            }

                            for (i, p) in meshes {
                                if let Some(indices) = &p.indices {
                                    let verticies = p
                                        .vertices
                                        .iter()
                                        .map(|v| ModelRaw {
                                            normal: v.normal.into(),
                                            tangent: v.tangent.clone().truncate().into(),
                                            position: v.position.into(),
                                            bitangent: Default::default(),
                                            tex_coords: v.tex_coord_0.into(),
                                        })
                                        .collect::<Vec<_>>();

                                    let mesh = MeshBuilder::new(self.device)
                                        .name("Some")
                                        .num_elements(indices.len() as u32)
                                        .material(p.index)
                                        .vertex_buffer_data(&verticies)
                                        .index_buffer_data(&indices)
                                        .vertex_buffer_binding(mesh_vertex_binding)
                                        .build()
                                        .unwrap();

                                    f_ms.push(mesh);
                                }
                            }
                        }
                    }
                }

                Ok(Model {
                    id,
                    meshes: f_ms,
                    materials: f_m,
                    bind_group_layout,
                })
            }
        }
    }
}

impl<'a> ModelBuilder<'a> {
    pub fn file(mut self, file: ModelFile) -> Self {
        self.file = Some(file);
        self
    }

    pub fn mesh_vertex_binding(mut self, mesh_vertex_binding: u32) -> Self {
        self.mesh_vertex_binding = Some(mesh_vertex_binding);
        self
    }

    pub fn mr_texture_params(mut self, tp: TextureParams) -> Self {
        self.mr = Some(tp);
        self
    }

    pub fn diffuse_texture_params(mut self, tp: TextureParams) -> Self {
        self.diffuse = Some(tp);
        self
    }

    pub fn normal_texture_params(mut self, tp: TextureParams) -> Self {
        self.normal = Some(tp);
        self
    }

    pub fn occlusion_texture_params(mut self, tp: TextureParams) -> Self {
        self.occlusion = Some(tp);
        self
    }

    pub fn emissive_texture_params(mut self, tp: TextureParams) -> Self {
        self.emissive = Some(tp);
        self
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, VertexLayout)]
#[attributes("Vertex")]
#[attributes("0 => Float32x3, 1 => Float32x2, 2 => Float32x3, 3 => Float32x3, 4 => Float32x3")]
struct ModelRaw {
    position: [f32; 3],
    tex_coords: [f32; 2],
    normal: [f32; 3],
    tangent: [f32; 3],
    bitangent: [f32; 3],
}
