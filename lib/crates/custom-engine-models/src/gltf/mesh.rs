use std::path::Path;

use collision::{Aabb, Aabb3, Union};

use crate::gltf::{Document, Primitive, Root};

#[derive(Debug, Clone)]
pub struct Mesh {
    pub index: usize,
    pub name: Option<String>,

    pub primitives: Vec<Primitive>,
    pub bounds: Aabb3<f32>,
}

impl Mesh {
    pub fn new(
        g_mesh: &gltf::Mesh<'_>,
        root: &mut Root,
        document: &Document,
        base_path: &Path,
    ) -> Mesh {
        let primitives: Vec<Primitive> = {
            let mut primitives = vec![];
            for p in g_mesh.primitives() {
                match Primitive::new(&p, root, g_mesh, document, base_path) {
                    Ok(m) => primitives.push(m),
                    Err(e) => panic!("Mesh new: {e}"),
                }
            }

            primitives
        };

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
