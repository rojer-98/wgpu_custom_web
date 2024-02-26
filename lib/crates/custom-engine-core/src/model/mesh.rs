use log::debug;

use crate::{
    buffer::{Buffer, BufferBuilder},
    errors::CoreError,
    traits::Builder,
};

#[derive(Debug)]
pub struct Mesh {
    pub id: usize,
    pub name: String,

    pub num_elements: u32,
    pub material: usize,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl Mesh {
    pub fn index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }
}

#[derive(Debug)]
pub struct MeshBuilder<'a, T: bytemuck::Pod + bytemuck::Zeroable> {
    id: Option<usize>,
    name: Option<&'a str>,
    index_buffer_data: Option<&'a [u32]>,
    vertex_buffer_data: Option<&'a [T]>,
    vertex_buffer_binding: Option<u32>,
    material: Option<usize>,
    num_elements: Option<u32>,

    device: &'a wgpu::Device,
}

impl<'a, T: bytemuck::Pod + bytemuck::Zeroable> Builder<'a> for MeshBuilder<'a, T> {
    type Final = Mesh;

    fn new(device: &'a wgpu::Device) -> Self
    where
        Self: Sized,
    {
        Self {
            id: None,
            name: None,
            index_buffer_data: None,
            vertex_buffer_data: None,
            vertex_buffer_binding: None,
            num_elements: None,
            material: None,
            device,
        }
    }

    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self
    where
        Self: Sized,
    {
        Self {
            id: Some(id),
            name: None,
            index_buffer_data: None,
            vertex_buffer_data: None,
            vertex_buffer_binding: None,
            num_elements: None,
            material: None,
            device,
        }
    }

    fn build(self) -> Result<Self::Final, CoreError>
    where
        Self: Sized,
    {
        let id = self.id.unwrap_or_default();
        let mesh_name = format!("Mesh: {id}");

        let name = self.name.unwrap_or(&mesh_name);
        let num_elements = self.num_elements.unwrap_or_default();
        let material = self.material.unwrap_or_default();

        let vertex_buffer_binding = self.vertex_buffer_binding.unwrap_or_default();
        let index_buffer_data = self
            .index_buffer_data
            .ok_or(CoreError::EmptyIndexData(name.to_string()))?;
        let index_buffer = BufferBuilder::new(self.device)
            .label(&format!("Index buffer: {name}"))
            .usage(wgpu::BufferUsages::INDEX)
            .data(index_buffer_data)
            .build()?;

        let vertex_buffer_data = self
            .vertex_buffer_data
            .ok_or(CoreError::EmptyData(name.to_string()))?;
        let vertex_buffer = BufferBuilder::new(self.device)
            .label(&format!("Vertex buffer: {name}"))
            .usage(wgpu::BufferUsages::VERTEX)
            .binding(vertex_buffer_binding)
            .data(vertex_buffer_data)
            .build()?;
        let name = name.to_string();

        debug!(
            "
Build `{name}`:
    Number elements: {num_elements},
    Material id: {material},
            "
        );

        Ok(Mesh {
            id,
            name,
            vertex_buffer,
            index_buffer,
            num_elements,
            material,
        })
    }
}

impl<'a, T: bytemuck::Pod + bytemuck::Zeroable> MeshBuilder<'a, T> {
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn index_buffer_data(mut self, data: &'a [u32]) -> Self {
        self.index_buffer_data = Some(data);
        self
    }

    pub fn vertex_buffer_data(mut self, data: &'a [T]) -> Self {
        self.vertex_buffer_data = Some(data);
        self
    }

    pub fn material(mut self, material: usize) -> Self {
        self.material = Some(material);
        self
    }

    pub fn num_elements(mut self, num_elements: u32) -> Self {
        self.num_elements = Some(num_elements);
        self
    }

    pub fn vertex_buffer_binding(mut self, vertex_buffer_binding: u32) -> Self {
        self.vertex_buffer_binding = Some(vertex_buffer_binding);
        self
    }
}
