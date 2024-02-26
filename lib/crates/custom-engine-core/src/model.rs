pub mod material;
pub mod mesh;

use custom_engine_models::obj::ObjFile;

use cgmath::{Vector2, Vector3};
use log::{debug, error};

use crate::{
    bind_group::layout::{BindGroupLayout, BindGroupLayoutBuilder},
    errors::CoreError,
    model::{
        material::{Material, MaterialBuilder},
        mesh::{Mesh, MeshBuilder},
    },
    traits::{Builder, VertexLayout},
};

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
    obj_file: Option<ObjFile>,

    mesh_vertex_binding: Option<u32>,

    diffuse_view_binding: Option<u32>,
    diffuse_sampler_binding: Option<u32>,

    normal_view_binding: Option<u32>,
    normal_sampler_binding: Option<u32>,

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
            obj_file: None,
            normal_view_binding: None,
            normal_sampler_binding: None,
            diffuse_view_binding: None,
            diffuse_sampler_binding: None,
            mesh_vertex_binding: None,
            device,
        }
    }

    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            id: Some(id),
            obj_file: None,
            normal_view_binding: None,
            normal_sampler_binding: None,
            diffuse_view_binding: None,
            diffuse_sampler_binding: None,
            mesh_vertex_binding: None,
            device,
        }
    }

    fn build(self) -> Result<Self::Final, CoreError>
    where
        Self: Sized,
    {
        let id = self.id.unwrap_or_default();
        let model_name = format!("Model: {id}");

        let obj_file = self
            .obj_file
            .ok_or(CoreError::EmptyObjFile(model_name.clone()))?;

        let diffuse_view_binding = self.diffuse_view_binding.unwrap_or(0);
        let diffuse_sampler_binding = self.diffuse_sampler_binding.unwrap_or(1);
        let normal_view_binding = self.normal_view_binding.unwrap_or(2);
        let normal_sampler_binding = self.normal_sampler_binding.unwrap_or(3);

        let mesh_vertex_binding = self.mesh_vertex_binding.unwrap_or_default();

        let bgl_name = format!("Bind Group Layout of `{model_name}`");
        let bind_group_layout = BindGroupLayoutBuilder::new(self.device)
            .label(&bgl_name)
            .entries(wgpu::BindGroupLayoutEntry {
                binding: diffuse_view_binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            })
            .entries(wgpu::BindGroupLayoutEntry {
                binding: diffuse_sampler_binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            })
            .entries(wgpu::BindGroupLayoutEntry {
                binding: normal_view_binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            })
            .entries(wgpu::BindGroupLayoutEntry {
                binding: normal_sampler_binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            })
            .build()?;

        let mut materials = Vec::new();
        for (i, lm) in obj_file.materials {
            let texture_name = lm.material.name.to_string();
            debug!(
                "
Proceed material: `{texture_name}:{i}`:
            "
            );

            let diffuse_texture_data =
                &lm.files
                    .diffuse_texture
                    .ok_or(CoreError::EmptyData(format!(
                        "Diffuse texture: {:?}",
                        lm.material.diffuse_texture
                    )))?;
            let normal_texture_data = lm.files.normal_texture.as_ref().map(|d| d.as_slice());

            let material = MaterialBuilder::new(self.device)
                .diffuse_texture_data(Some(diffuse_texture_data))
                .diffuse_view_binding(diffuse_view_binding)
                .diffuse_sampler_binding(diffuse_sampler_binding)
                .normal_texture_data(normal_texture_data)
                .normal_view_binding(normal_view_binding)
                .normal_sampler_binding(normal_sampler_binding)
                .layout(&bind_group_layout)
                .build()?;

            materials.push(material)
        }

        let meshes = obj_file
            .models
            .into_iter()
            .map(|(_, m)| -> Result<Mesh, CoreError> {
                let mut vertices = (0..m.mesh.positions.len() / 3)
                    .map(|i| ModelRaw {
                        position: [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                        tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]].into(),
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
                    let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                    vertices[c[0] as usize].tangent =
                        (tangent + Vector3::from(vertices[c[0] as usize].tangent)).into();
                    vertices[c[1] as usize].tangent =
                        (tangent + Vector3::from(vertices[c[1] as usize].tangent)).into();
                    vertices[c[2] as usize].tangent =
                        (tangent + Vector3::from(vertices[c[2] as usize].tangent)).into();
                    vertices[c[0] as usize].bitangent =
                        (bitangent + Vector3::from(vertices[c[0] as usize].bitangent)).into();
                    vertices[c[1] as usize].bitangent =
                        (bitangent + Vector3::from(vertices[c[1] as usize].bitangent)).into();
                    vertices[c[2] as usize].bitangent =
                        (bitangent + Vector3::from(vertices[c[2] as usize].bitangent)).into();

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
}

impl<'a> ModelBuilder<'a> {
    pub fn obj_file(mut self, obj_file: ObjFile) -> Self {
        self.obj_file = Some(obj_file);
        self
    }

    pub fn normal_sampler_binding(mut self, normal_sampler_binding: u32) -> Self {
        self.normal_sampler_binding = Some(normal_sampler_binding);
        self
    }

    pub fn normal_view_binding(mut self, normal_view_binding: u32) -> Self {
        self.normal_view_binding = Some(normal_view_binding);
        self
    }

    pub fn diffuse_sampler_binding(mut self, diffuse_sampler_binding: u32) -> Self {
        self.diffuse_sampler_binding = Some(diffuse_sampler_binding);
        self
    }

    pub fn diffuse_view_binding(mut self, diffuse_view_binding: u32) -> Self {
        self.diffuse_view_binding = Some(diffuse_view_binding);
        self
    }

    pub fn mesh_vertex_binding(mut self, mesh_vertex_binding: u32) -> Self {
        self.mesh_vertex_binding = Some(mesh_vertex_binding);
        self
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelRaw {
    position: [f32; 3],
    tex_coords: [f32; 2],
    normal: [f32; 3],
    tangent: [f32; 3],
    bitangent: [f32; 3],
}

impl VertexLayout for ModelRaw {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3, 3 => Float32x3, 4 => Float32x3,];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem::size_of;

        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBUTES,
        }
    }
}
