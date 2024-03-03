use cgmath::dot;
use cgmath::InnerSpace;
use cgmath::Vector3;
use derive_more::Constructor;

use crate::{primitives::Vertex, traits::Component};

#[derive(Debug, Clone)]
pub struct Triangle {
    points: [Vector3<f32>; 3],
    color: Vector3<f32>,
}

impl Triangle {
    pub fn new(points: [Vector3<f32>; 3], color: Vector3<f32>) -> Self {
        Self { points, color }
    }

    pub fn point_inside(&self, point: &Vector3<f32>) -> bool {
        let [mut a, mut b, mut c] = self.points.clone();

        a -= *point;
        b -= *point;
        c -= *point;

        let u = b.cross(a);
        let v = c.cross(b);
        let w = a.cross(c);

        if u.dot(v) < 0. {
            return false;
        }

        if u.dot(w) < 0. {
            return false;
        }

        true
    }
}

mod tests {
    use cgmath::Vector3;

    use super::Triangle;

    #[test]
    fn point_inside() {
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
    }
}
