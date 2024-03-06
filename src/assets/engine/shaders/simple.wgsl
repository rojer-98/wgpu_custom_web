struct VertexInput {
    @location(0) @interpolate(flat) controls: vec4<u32>,
    @location(1) position: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) tex_coord: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) @interpolate(flat) is_click: u32,
}

struct Controls {
    @location(0) size: vec4<f32>,
}

@group(0) @binding(0) 
var<uniform> controls: Controls;

fn to_shader_coordX_naga_oil_mod_XONUW24DMMVPWM5LOMN2GS33OOMX(model_position: vec3<f32>, size: vec4<f32>) -> vec3<f32> {
    var position_1: vec3<f32>;

    let half_w: f32 = (size.x / 2f);
    let half_h: f32 = (size.y / 2f);
    if (model_position.x > half_w) {
        position_1.x = ((model_position.x / half_w) - 1f);
    } else {
        position_1.x = -((1f - (model_position.x / half_w)));
    }
    if (model_position.y > half_h) {
        position_1.y = -(((model_position.y / half_h) - 1f));
    } else {
        position_1.y = (1f - (model_position.y / half_h));
    }
    let _e35: vec3<f32> = position_1;
    return _e35;
}

@vertex 
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var position: vec3<f32>;

    let is_click: u32 = (model.controls.x & 1u);
    let converted: u32 = ((model.controls.x >> 1u) & 1u);
    if bool(converted) {
        position = model.position;
    } else {
        let _e17: vec4<f32> = controls.size;
        let _e18: vec3<f32> = to_shader_coordX_naga_oil_mod_XONUW24DMMVPWM5LOMN2GS33OOMX(model.position, _e17);
        position = _e18;
    }
    out.color = model.color;
    let _e23: vec3<f32> = position;
    out.clip_position = vec4<f32>(_e23, 1f);
    out.is_click = is_click;
    let _e27: VertexOutput = out;
    return _e27;
}

@fragment 
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if bool(in.is_click) {
        let b_color: vec4<f32> = vec4<f32>(in.color, 0.3f);
        let s_color: vec4<f32> = vec4<f32>(0.235f, 0.564f, 1f, 0f);
        return (b_color + s_color);
    } else {
        return vec4<f32>(in.color, 1f);
    }
}
