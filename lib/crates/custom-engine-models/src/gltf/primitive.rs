use std::{path::Path, rc::Rc};

use cgmath::{Vector2, Vector3, Vector4, Zero};
use collision::Aabb3;
use log::warn;

use crate::gltf::{Document, GltfMesh, GltfPrimitive, Material, Root};

#[derive(Debug)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tangent: Vector4<f32>,
    pub tex_coord_0: Vector2<f32>,
    pub tex_coord_1: Vector2<f32>,
    pub color_0: Vector4<f32>,
    pub joints_0: Vector4<u16>,
    pub weights_0: Vector4<f32>,
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Vector3::zero(),
            normal: Vector3::zero(),
            tangent: Vector4::zero(),
            tex_coord_0: Vector2::zero(),
            tex_coord_1: Vector2::zero(),
            color_0: Vector4::zero(),
            joints_0: Vector4::zero(),
            weights_0: Vector4::zero(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Primitive {
    pub index: usize,
    pub bounds: Aabb3<f32>,
}

impl Primitive {
    pub fn new<'a>(
        gltf_primitive: &'a GltfPrimitive<'a>,
        root: &'a mut Root,
        mesh: &'a GltfMesh<'a>,
        doc: &'a Document,
        base_path: &'a Path,
    ) -> Self {
        let index = gltf_primitive.index();
        let mesh_index = mesh.index();

        let reader = gltf_primitive.reader(|b| Some(&doc.buffers[b.index()]));
        let positions = {
            let iter = reader.read_positions().unwrap_or_else(|| {
                panic!(
                    "primitives must have the POSITION attribute (mesh: {mesh_index}, primitive: {index})",
                )
            });
            iter.collect::<Vec<_>>()
        };

        let bounds = gltf_primitive.bounding_box();
        let bounds = Aabb3 {
            min: bounds.min.into(),
            max: bounds.max.into(),
        };

        let mut vertices: Vec<Vertex> = positions
            .into_iter()
            .map(|position| Vertex {
                position: Vector3::from(position),
                ..Vertex::default()
            })
            .collect();

        if let Some(normals) = reader.read_normals() {
            for (i, normal) in normals.enumerate() {
                vertices[i].normal = Vector3::from(normal);
            }
        }
        if let Some(tangents) = reader.read_tangents() {
            for (i, tangent) in tangents.enumerate() {
                vertices[i].tangent = Vector4::from(tangent);
            }
        }

        let mut tex_coord_set = 0;
        while let Some(tex_coords) = reader.read_tex_coords(tex_coord_set) {
            if tex_coord_set > 1 {
                warn!(
                    "Ignoring texture coordinate set {tex_coord_set}, \
                        only supporting 2 sets at the moment. (mesh: {mesh_index}, primitive: {index})",
                );
                tex_coord_set += 1;
                continue;
            }
            for (i, tex_coord) in tex_coords.into_f32().enumerate() {
                match tex_coord_set {
                    0 => vertices[i].tex_coord_0 = Vector2::from(tex_coord),
                    1 => vertices[i].tex_coord_1 = Vector2::from(tex_coord),
                    _ => unreachable!(),
                }
            }
            tex_coord_set += 1;
        }
        if let Some(colors) = reader.read_colors(0) {
            let colors = colors.into_rgba_f32();
            for (i, c) in colors.enumerate() {
                vertices[i].color_0 = c.into();
            }
        }
        if reader.read_colors(1).is_some() {
            warn!("Ignoring further color attributes, only supporting COLOR_0. (mesh: {mesh_index}, primitive: {index})");
        }
        if let Some(joints) = reader.read_joints(0) {
            for (i, joint) in joints.into_u16().enumerate() {
                vertices[i].joints_0 = joint.into();
            }
        }
        if reader.read_joints(1).is_some() {
            warn!("Ignoring further joint attributes, only supporting JOINTS_0. (mesh: {mesh_index}, primitive: {index})");
        }
        if let Some(weights) = reader.read_weights(0) {
            for (i, weights) in weights.into_f32().enumerate() {
                vertices[i].weights_0 = weights.into();
            }
        }
        if reader.read_weights(1).is_some() {
            warn!("Ignoring further weight attributes, only supporting WEIGHTS_0. (mesh: {mesh_index}, primitive: {index})");
        }

        let _indices = reader
            .read_indices()
            .map(|read_indices| read_indices.into_u32().collect::<Vec<_>>());

        let _mode = gltf_primitive.mode().as_gl_enum();
        let g_material = gltf_primitive.material();
        let mut material = None;
        if let Some(mat) = root
            .materials
            .iter()
            .find(|m| (***m).index == g_material.index())
        {
            material = Rc::clone(mat).into()
        }

        if material.is_none() {
            // no else due to borrow checker madness
            let mat = Rc::new(Material::from_gltf(&g_material, root, doc, base_path));
            root.materials.push(Rc::clone(&mat));
            material = Some(mat);
        };
        let material = material.unwrap();

        Self { index, bounds }
    }
}
