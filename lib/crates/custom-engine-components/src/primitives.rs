use custom_engine_core::traits::VertexLayout;
use custom_engine_derive::VertexLayout;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, VertexLayout)]
#[attributes("Vertex")]
#[attributes("0 => Uint32x4, 1 => Float32x3, 2 => Float32x3, 3 => Float32x2")]
pub struct Vertex {
    pub(crate) controls: [u32; 4],
    pub(crate) position: [f32; 3],
    pub(crate) _pad0: f32,
    pub(crate) color: [f32; 3],
    pub(crate) _pad1: f32,
    pub(crate) tex_coords: [f32; 2],
    pub(crate) _pad2: [f32; 2],
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, VertexLayout)]
#[attributes("Vertex")]
#[attributes("3 => Float32x3, 4 => Float32x3, 5 => Float32x3")]
pub struct VertexAdditional {
    pub(crate) normal: [f32; 3],
    pub(crate) _pad0: f32,
    pub(crate) tangent: [f32; 3],
    pub(crate) _pad1: f32,
    pub(crate) bitangent: [f32; 3],
    pub(crate) _pad2: f32,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, VertexLayout)]
#[attributes("Vertex")]
#[attributes("10 => Float32x4, 11 => Float32x4, 12 => Float32x4, 13 => Float32x4,")]
pub struct MatrixModel {
    pub(crate) _0: [f32; 4],
    pub(crate) _1: [f32; 4],
    pub(crate) _2: [f32; 4],
    pub(crate) _3: [f32; 4],
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, VertexLayout)]
#[attributes("Vertex")]
#[attributes("14 => Float32x3, 15 => Float32x3, 16 => Float32x3")]
pub struct MatrixNormal {
    pub(crate) _0: [f32; 3],
    pub(crate) _pad0: f32,
    pub(crate) _1: [f32; 3],
    pub(crate) _pad1: f32,
    pub(crate) _2: [f32; 3],
    pub(crate) _pad2: f32,
}
