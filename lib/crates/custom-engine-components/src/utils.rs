pub fn to_shader_coords(position: (f64, f64), size: (u32, u32)) -> (f32, f32) {
    let (w, h) = size;
    let (x, y) = (position.0 as f32, position.1 as f32);
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

    (out_x, out_y)
}
