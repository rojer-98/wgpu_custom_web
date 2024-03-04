struct VertexInput {
    @location(0) @interpolate(flat) controls: vec4<u32>,
    @location(1) position: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) tex_coord: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) @interpolate(flat) is_click: vec4<u32>,
}

struct Controls {
    @location(0) size: vec4<f32>,
}

@group(0) @binding(0) 
var<uniform> controls: Controls;

fn to_shader_coordX_naga_oil_mod_XONUW24DMMVPWM5LOMN2GS33OOMX(model_position: vec3<f32>, size: vec4<f32>) -> vec3<f32> {
    var position: vec3<f32>;

    let half_w: f32 = (size.x / 2f);
    let half_h: f32 = (size.y / 2f);
    if (model_position.x > half_w) {
        position.x = ((model_position.x / half_w) - 1f);
    } else {
        position.x = -((1f - (model_position.x / half_w)));
    }
    if (model_position.y > half_h) {
        position.y = -(((model_position.y / half_h) - 1f));
    } else {
        position.y = (1f - (model_position.y / half_h));
    }
    let _e35: vec3<f32> = position;
    return _e35;
}

@vertex 
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.color = model.color;
    let _e8: vec4<f32> = controls.size;
    let _e9: vec3<f32> = to_shader_coordX_naga_oil_mod_XONUW24DMMVPWM5LOMN2GS33OOMX(model.position, _e8);
    out.clip_position = vec4<f32>(_e9, 1f);
    out.is_click = model.controls;
    let _e14: VertexOutput = out;
    return _e14;
}

@fragment 
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.is_click.x == 1u) {
        return vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        return vec4<f32>(in.color, 1f);
    }
}
