use std::path::Path;

use collision::{Aabb, Aabb3, Union};

use crate::gltf::{Document, GltfMesh, Primitive, Root};

#[derive(Debug, Clone)]
pub struct Mesh {
    pub index: usize,
    pub name: Option<String>,

    pub primitives: Vec<Primitive>,
    pub bounds: Aabb3<f32>,
}

impl Mesh {
    pub fn new(
        g_mesh: &GltfMesh<'_>,
        root: &mut Root,
        document: &Document,
        base_path: &Path,
    ) -> Mesh {
        let primitives: Vec<Primitive> = g_mesh
            .primitives()
            .map(|g_prim| Primitive::new(&g_prim, root, g_mesh, document, base_path))
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
