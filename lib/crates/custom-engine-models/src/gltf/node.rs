use std::{path::Path, rc::Rc};

use cgmath::{Matrix4, Quaternion, SquareMatrix, Vector3};
use collision::{Aabb, Aabb3, Union};

use crate::gltf::{Camera, Document, Mesh, Root};

#[derive(Debug)]
pub struct Node {
    pub index: usize,
    pub children: Vec<usize>,
    pub mesh: Option<Rc<Mesh>>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    pub translation: Vector3<f32>,
    pub camera: Option<Rc<Camera>>,
    pub name: Option<String>,

    pub final_transform: Matrix4<f32>,
    pub bounds: Aabb3<f32>,
}

impl Node {
    pub async fn new(
        g_node: &gltf::Node<'_>,
        root: &mut Root,
        document: &Document,
        base_path: &Path,
    ) -> Node {
        let (trans, rot, scale) = g_node.transform().decomposed();
        let r = rot;
        let rotation = Quaternion::new(r[3], r[0], r[1], r[2]); // NOTE: different element order!

        let mut mesh = None;
        if let Some(g_mesh) = g_node.mesh() {
            if let Some(existing_mesh) =
                root.meshes.iter().find(|mesh| mesh.index == g_mesh.index())
            {
                mesh = Some(Rc::clone(existing_mesh));
            }

            if mesh.is_none() {
                mesh = Some(Rc::new(Mesh::new(&g_mesh, root, document, base_path).await));

                root.meshes.push(mesh.clone().unwrap());
            }
        }
        let children: Vec<_> = g_node.children().map(|g_node| g_node.index()).collect();

        Node {
            index: g_node.index(),
            children,
            mesh,
            rotation,
            scale: scale.into(),
            translation: trans.into(),
            camera: g_node.camera().as_ref().map(|c| Rc::new(Camera::new(c))),
            name: g_node.name().map(|s| s.into()),

            final_transform: Matrix4::identity(),

            bounds: Aabb3::zero(),
        }
    }

    pub fn update_transform(&mut self, root: &mut Root, parent_transform: &Matrix4<f32>) {
        self.final_transform = *parent_transform
            * Matrix4::from_translation(self.translation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
            * Matrix4::from(self.rotation);

        self.children.iter().for_each(|id| {
            root.unsafe_get_node_mut(*id)
                .update_transform(root, &self.final_transform);
        })
    }

    /// Should be called after update_transforms
    pub fn update_bounds(&mut self, root: &mut Root) {
        self.bounds = Aabb3::zero();
        if let Some(ref mesh) = self.mesh {
            self.bounds = mesh.bounds.transform(&self.final_transform);
        }

        self.children.iter().for_each(|id| {
            let node = root.unsafe_get_node_mut(*id);
            node.update_bounds(root);

            self.bounds = self.bounds.union(&node.bounds);
        });
    }
}
