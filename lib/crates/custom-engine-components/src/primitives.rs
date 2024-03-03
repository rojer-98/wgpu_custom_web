use custom_engine_core::traits::VertexLayout;
use custom_engine_derive::VertexLayout;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, VertexLayout)]
#[attributes("Vertex")]
#[attributes("0 => Float32x3, 1 => Float32x3, 2 => Float32x2")]
pub struct Vertex {
    position: [f32; 3],
    _pad0: f32,
    color: [f32; 3],
    _pad1: f32,
    tex_coords: [f32; 2],
    _pad2: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, VertexLayout)]
#[attributes("Vertex")]
#[attributes("3 => Float32x3, 4 => Float32x3, 5 => Float32x3")]
pub struct VertexAdditional {
    normal: [f32; 3],
    _pad0: f32,
    tangent: [f32; 3],
    _pad1: f32,
    bitangent: [f32; 3],
    _pad2: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, VertexLayout)]
#[attributes("Vertex")]
#[attributes("10 => Float32x4, 11 => Float32x4, 12 => Float32x4, 13 => Float32x4,")]
pub struct MatrixModel {
    _0: [f32; 4],
    _1: [f32; 4],
    _2: [f32; 4],
    _3: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, VertexLayout)]
#[attributes("Vertex")]
#[attributes("14 => Float32x3, 15 => Float32x3, 16 => Float32x3")]
pub struct MatrixNormal {
    _0: [f32; 3],
    _pad0: f32,
    _1: [f32; 3],
    _pad1: f32,
    _2: [f32; 3],
    _pad2: f32,
}
