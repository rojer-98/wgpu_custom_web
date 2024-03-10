use cgmath::{Matrix4, SquareMatrix};
use collision::{Aabb, Aabb3, Union};

use crate::gltf::Root;

#[derive(Debug)]
pub struct Scene {
    pub name: Option<String>,
    pub nodes: Vec<usize>,
    pub bounds: Aabb3<f32>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            name: None,
            nodes: vec![],
            bounds: Aabb3::<f32>::zero(),
        }
    }
}

impl Scene {
    pub fn new(g_scene: &gltf::Scene<'_>, root: &mut Root) -> Scene {
        let mut scene = Scene {
            name: g_scene.name().map(|s| s.to_owned()),
            nodes: g_scene.nodes().map(|g_node| g_node.index()).collect(),

            ..Default::default()
        };

        let root_transform = Matrix4::identity();
        scene.nodes.iter().for_each(|node_id| {
            let node = root.unsafe_get_node_mut(*node_id);

            node.update_transform(root, &root_transform);
            node.update_bounds(root);

            scene.bounds = scene.bounds.union(&node.bounds);
        });

        scene
    }
}
