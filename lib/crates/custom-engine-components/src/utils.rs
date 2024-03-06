use cgmath::Vector3;

pub fn to_shader_coords<T: Into<Vector3<f32>>>(position: T, size: (u32, u32)) -> Vector3<f32> {
    let position = position.into();

    let (w, h) = size;
    let (x, y) = (position.x as f32, position.y as f32);
    let (half_w, half_h) = (w as f32 / 2., h as f32 / 2.);

    let out_x = if x > half_w {
        x / half_w - 1.
    } else {
        -(1. - x / half_w)
    };

    let out_y = if y > half_h {
        -(y / half_h - 1.)
    } else {
        1. - y / half_h
    };

    Vector3::new(out_x, out_y, position.z)
}
