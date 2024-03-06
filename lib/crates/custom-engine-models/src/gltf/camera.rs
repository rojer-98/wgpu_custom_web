use cgmath::{perspective, Deg, Matrix4, Rad, Zero};
use gltf::{self, camera::Projection};

#[derive(Debug, Clone)]
pub struct OrthographicCamera {
    pub index: usize,
    pub name: Option<String>,

    projection_matrix: Matrix4<f32>,
    znear: f32,
    zfar: f32,

    xmag: f32,
    ymag: f32,
}

#[derive(Debug, Clone)]
pub struct PerspectiveCamera {
    pub index: usize,
    pub name: Option<String>,

    projection_matrix: Matrix4<f32>,
    znear: f32,
    zfar: Option<f32>,

    fovy: Deg<f32>,
    aspect_ratio: f32,
}

#[derive(Debug, Clone)]
pub enum Camera {
    Orthographic(OrthographicCamera),
    Perspective(PerspectiveCamera),
}

impl Camera {
    pub fn description(&self) -> &str {
        use Camera::*;

        match self {
            Orthographic(_) => "Orthographic",
            Perspective(_) => "Perspective",
        }
    }

    pub fn update_projection_matrix(&mut self) {
        use Camera::*;

        match self {
            Orthographic(o) => {
                let r = o.xmag;
                let t = o.ymag;
                let f = o.zfar;
                let n = o.znear;
                o.projection_matrix = Matrix4::new(
                    1.0 / r,
                    0.0,
                    0.0,
                    0.0, // NOTE: first column!
                    0.0,
                    1.0 / t,
                    0.0,
                    0.0, // 2nd
                    0.0,
                    0.0,
                    2.0 / (n - f),
                    0.0, // 3rd
                    0.0,
                    0.0,
                    (f + n) / (n - f),
                    1.0, // 4th
                );
            }
            Perspective(p) => {
                if let Some(zfar) = p.zfar {
                    p.projection_matrix = perspective(p.fovy, p.aspect_ratio, p.znear, zfar);
                } else {
                    let a = p.aspect_ratio;
                    let y = Rad::from(p.fovy).0;
                    let n = p.znear;

                    p.projection_matrix = Matrix4::new(
                        1.0 / (a * (0.5 * y).tan()),
                        0.0,
                        0.0,
                        0.0, // NOTE: first column!
                        0.0,
                        1.0 / (0.5 * y).tan(),
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        -1.0,
                        -1.0,
                        0.0,
                        0.0,
                        -2.0 * n,
                        0.0,
                    );
                }
            }
        }
    }
}

impl<'a> From<&gltf::Camera<'a>> for Camera {
    fn from(gltf_camera: &gltf::Camera<'a>) -> Self {
        let index = gltf_camera.index();
        let name = gltf_camera.name().map(|n| n.to_owned());
        let projection_matrix = Matrix4::zero();

        let mut camera = match gltf_camera.projection() {
            Projection::Perspective(p) => {
                let znear = p.znear().max(0.0001);
                let zfar = p.zfar();
                let fovy = Deg::from(Rad(p.yfov()));
                let aspect_ratio = p.aspect_ratio().unwrap_or(1.);

                Camera::Perspective(PerspectiveCamera {
                    index,
                    name,
                    projection_matrix,
                    znear,
                    zfar,
                    fovy,
                    aspect_ratio,
                })
            }
            Projection::Orthographic(o) => {
                let xmag = o.xmag();

                let ymag = o.ymag();
                let znear = o.znear();
                let zfar = o.zfar();

                Camera::Orthographic(OrthographicCamera {
                    index,
                    name,
                    projection_matrix,
                    znear,
                    zfar,
                    xmag,
                    ymag,
                })
            }
        };
        camera.update_projection_matrix();

        camera
    }
}
