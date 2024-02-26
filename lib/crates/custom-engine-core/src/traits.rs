use crate::{errors::CoreError, worker::Worker};

use winit::event::WindowEvent;

pub trait VertexLayout
where
    Self: bytemuck::Zeroable + bytemuck::Pod,
{
    const ATTRIBUTES: &'static [wgpu::VertexAttribute];

    fn desc() -> wgpu::VertexBufferLayout<'static>;
    fn to_bytes(&self) -> Vec<u8> {
        bytemuck::cast_vec(vec![*self])
    }
}

pub trait Builder<'a> {
    type Final;

    fn new(device: &'a wgpu::Device) -> Self
    where
        Self: Sized;
    fn new_indexed(device: &'a wgpu::Device, id: usize) -> Self
    where
        Self: Sized;

    fn build(self) -> Result<Self::Final, CoreError>
    where
        Self: Sized;
}

pub trait RenderWorker {
    fn init(w: &mut Worker<'_>) -> Result<Self, CoreError>
    where
        Self: Sized;
    fn reinit(&mut self, w: &mut Worker<'_>) -> Result<(), CoreError>
    where
        Self: Sized;

    fn update(&mut self, w: &mut Worker<'_>, event: &WindowEvent) -> Result<(), CoreError>;
    fn render(&mut self, w: &mut Worker<'_>) -> Result<(), CoreError>;
}
