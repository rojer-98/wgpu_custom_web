use std::{path::Path, rc::Rc};

use crate::gltf::{
    camera::Camera, document::Document, material::Material, mesh::Mesh, node::Node,
    texture::Texture,
};

#[derive(Default, Debug)]
pub struct Root {
    pub nodes: Vec<Node>,
    pub meshes: Vec<Rc<Mesh>>,
    pub textures: Vec<Rc<Texture>>,
    pub materials: Vec<Rc<Material>>,
    pub camera_nodes: Vec<Rc<Camera>>,
}

impl Root {
    pub async fn new(document: &Document, base_path: &Path) -> Self {
        let mut root = Root::default();

        root.nodes = {
            let mut nodes = vec![];

            for n in document.inner.nodes() {
                nodes.push(Node::new(&n, &mut root, document, base_path).await);
            }

            nodes
        };
        root.camera_nodes = root
            .nodes
            .iter()
            .filter_map(|node| {
                if let Some(c) = node.camera.as_ref() {
                    Some(c.clone())
                } else {
                    None
                }
            })
            .collect();
        root
    }

    /// Get a mutable reference to a node without borrowing `Self` or `Self::nodes`.
    /// Safe for tree traversal (visiting each node ONCE and NOT keeping a reference)
    /// as long as the gltf is valid, i.e. the scene actually is a tree.
    pub fn unsafe_get_node_mut(&mut self, index: usize) -> &'static mut Node {
        unsafe { &mut *(&mut self.nodes[index] as *mut Node) }
    }

    pub fn unsafe_get_node(&self, index: usize) -> &'static Node {
        unsafe { &*(&self.nodes[index] as *const Node) }
    }
}
