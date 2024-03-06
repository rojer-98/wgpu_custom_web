use std::path::Path;

use collision::{Aabb, Aabb3, Union};

use crate::gltf::{document::Document, primitive::Primitive, root::Root};

#[derive(Debug, Clone)]
pub struct Mesh {
    pub index: usize, // glTF index
    pub primitives: Vec<Primitive>,
    // TODO: weights
    // pub weights: Vec<Rc<?>>
    pub name: Option<String>,

    pub bounds: Aabb3<f32>,
}

impl Mesh {
    pub fn from_gltf(
        g_mesh: &gltf::Mesh<'_>,
        root: &mut Root,
        document: &Document,
        base_path: &Path,
    ) -> Mesh {
        let primitives: Vec<Primitive> = g_mesh
            .primitives()
            .enumerate()
            .map(|(i, g_prim)| Primitive::new(&g_prim, root, g_mesh, document))
            .collect();

        let bounds = primitives
            .iter()
            .fold(Aabb3::zero(), |bounds, prim| prim.bounds.union(&bounds));

        Mesh {
            index: g_mesh.index(),
            primitives,
            name: g_mesh.name().map(|s| s.into()),
            bounds,
        }
    }
}
