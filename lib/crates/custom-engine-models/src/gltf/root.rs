use std::{path::Path, rc::Rc};

use crate::gltf::{
    document::Document, material::Material, mesh::Mesh, node::Node, texture::Texture,
};

#[derive(Default, Debug)]
pub struct Root {
    pub nodes: Vec<Node>,
    pub meshes: Vec<Rc<Mesh>>,
    pub textures: Vec<Rc<Texture>>,
    pub materials: Vec<Rc<Material>>,
    pub camera_nodes: Vec<usize>,
}

impl Root {
    pub fn new(document: &Document, base_path: &Path) -> Self {
        let mut root = Root::default();
        let nodes = document
            .inner
            .nodes()
            .map(|g_node| Node::new(&g_node, &mut root, document, base_path))
            .collect();
        root.nodes = nodes;
        root.camera_nodes = root
            .nodes
            .iter()
            .filter(|node| node.camera.is_some())
            .map(|node| node.index)
            .collect();
        root
    }

    /// Get a mutable reference to a node without borrowing `Self` or `Self::nodes`.
    /// Safe for tree traversal (visiting each node ONCE and NOT keeping a reference)
    /// as long as the gltf is valid, i.e. the scene actually is a tree.
    pub fn unsafe_get_node_mut(&mut self, index: usize) -> &'static mut Node {
        unsafe { &mut *(&mut self.nodes[index] as *mut Node) }
    }

    /// Note: index refers to the vec of camera node indices!
    pub fn get_camera_node(&self, index: usize) -> &Node {
        &self.nodes[self.camera_nodes[index]]
    }
}
