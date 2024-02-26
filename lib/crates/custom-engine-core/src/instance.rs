use cgmath::{Deg, InnerSpace, Matrix4, Quaternion, Rotation3, Vector3, Zero};

use crate::traits::VertexLayout;

#[derive(Debug)]
pub struct Instances(Vec<Instance>);

impl Instances {
    pub fn new(space_between: f32, num_instances_per_row: u32) -> Self {
        Self(
            (0..num_instances_per_row)
                .flat_map(|z| {
                    (0..num_instances_per_row)
                        .map(move |x| Instance::new(x, z, space_between, num_instances_per_row))
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn data(&self) -> Vec<InstanceRaw> {
        self.0.iter().map(Instance::data).collect::<Vec<_>>()
    }

    #[inline]
    pub fn get_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        InstanceRaw::desc()
    }
}

#[derive(Debug)]
pub struct Instance {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
}

impl Instance {
    pub fn new(x: u32, z: u32, space_between: f32, num_instances_per_row: u32) -> Self {
        let x = space_between * (x as f32 - num_instances_per_row as f32 / 2.0);
        let z = space_between * (z as f32 - num_instances_per_row as f32 / 2.0);

        let position = Vector3 { x, y: 0.0, z };

        let rotation = if position.is_zero() {
            Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0))
        } else {
            Quaternion::from_axis_angle(position.normalize(), Deg(45.0))
        };

        Self { position, rotation }
    }

    pub fn data(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Matrix4::from_translation(self.position) * Matrix4::from(self.rotation)).into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    #[allow(dead_code)]
    model: [[f32; 4]; 4],
}

impl VertexLayout for InstanceRaw {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] =
        &wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem::size_of;

        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}
