use cgmath::{Deg, InnerSpace, Matrix3, Matrix4, Quaternion, Rotation3, Vector3, Zero};

use custom_engine_derive::VertexLayout;

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
            normal: Matrix3::from(self.rotation).into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, VertexLayout)]
#[attributes("Instance")]
#[attributes("5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4, 9 => Float32x3, 10 => Float32x3, 11 => Float32x3")]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}
