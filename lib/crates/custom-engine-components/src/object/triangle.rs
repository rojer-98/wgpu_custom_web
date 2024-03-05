use std::array::from_fn;

use cgmath::{InnerSpace, Vector3, Vector4};
use derive_more::{Deref, DerefMut};

use custom_engine_core::traits::VertexLayout;

use crate::{object::metadata::ControlFlags, primitives::Vertex, to_shader_coords};

#[derive(Debug, Deref, DerefMut)]
pub struct Triangles {
    #[deref]
    #[deref_mut]
    inner: Vec<Triangle>,
}

impl Triangles {
    pub fn click<T: Into<Vector3<f32>>>(&mut self, point: T) {
        let point = point.into();

        self.iter_mut().for_each(|p| p.click(&point));
    }

    pub fn move_to<T: Into<Vector3<f32>>>(&mut self, m: T) {
        let m = m.into();

        self.iter_mut()
            .filter(|t| (t.controls.x & ControlFlags::Click.to_u32()) == 1)
            .for_each(|t| {
                t.points.iter_mut().for_each(|p| {
                    *p += m;
                })
            });
    }

    pub fn to_data(&self) -> Vec<Vertex> {
        self.iter()
            .map(|t| t.to_data().to_vec())
            .flatten()
            .collect()
    }
}

impl From<Vec<Triangle>> for Triangles {
    fn from(value: Vec<Triangle>) -> Self {
        Self { inner: value }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    points: [Vector3<f32>; 3],
    color: Vector3<f32>,
    controls: Vector4<u32>,
}

impl Triangle {
    pub fn new<T: Into<Vector3<f32>>>(points: [T; 3], color: T) -> Self {
        let mut converted_points = points.into_iter().map(|p| p.into());

        Self {
            points: from_fn(|_| converted_points.next().unwrap()),
            color: color.into(),
            controls: Vector4::new(0, 0, 0, 0),
        }
    }

    pub fn to_data(&self) -> [Vertex; 3] {
        let mut verts = [Vertex::default(), Vertex::default(), Vertex::default()];

        for (i, p) in self.points.iter().enumerate() {
            verts[i] = Vertex {
                controls: self.controls.into(),
                color: self.color.into(),
                position: (*p).into(),
                ..Default::default()
            };
        }

        verts
    }

    pub fn to_data_converted(&self, size: (u32, u32)) -> [Vertex; 3] {
        let mut verts = [Vertex::default(), Vertex::default(), Vertex::default()];

        for (i, p) in self.points.iter().enumerate() {
            let position = to_shader_coords(*p, size);
            let mut controls = self.controls;
            controls.x &= ControlFlags::Convert.to_u32();

            verts[i] = Vertex {
                controls: controls.into(),
                color: self.color.into(),
                position: position.into(),
                ..Default::default()
            };
        }

        verts
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        Vertex::desc()
    }

    pub fn click<'a, T: Into<&'a Vector3<f32>>>(&mut self, point: T) {
        if self.point_inside(point.into()) {
            self.controls.x ^= ControlFlags::Click.to_u32();
        }
    }

    fn point_inside<'a, T: Into<&'a Vector3<f32>>>(&self, point: T) -> bool {
        let point = point.into();
        let [mut a, mut b, mut c] = self.points.clone();

        a -= *point;
        b -= *point;
        c -= *point;

        let u = b.cross(a);
        let v = c.cross(b);
        let w = a.cross(c);

        u.dot(v) > 0. && u.dot(w) > 0.
    }
}

mod tests {
    #[test]
    fn point_inside() {
        use cgmath::Vector3;

        use super::Triangle;

        let points = [
            Vector3::new(40., 20., 1.),
            Vector3::new(100., 200., 1.),
            Vector3::new(150., 30., 1.),
        ];
        let color = Vector3::new(0., 0., 0.);
        let t = Triangle::new(points, color);

        let center = Vector3::new(96., 50., 1.);

        assert_eq!(true, t.point_inside(&center));
        assert_eq!(false, t.point_inside(&Vector3::new(20., 300., 1.)));

        let points = [[40., 20., 1.], [100., 200., 1.], [150., 30., 1.]];
        let color = [0., 0., 0.];
        let t = Triangle::new(points, color);

        let center = Vector3::new(96., 50., 1.);

        assert_eq!(true, t.point_inside(&center));
        assert_eq!(false, t.point_inside(&Vector3::new(20., 300., 1.)));
    }
}
