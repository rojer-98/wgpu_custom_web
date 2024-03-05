#import simple_functions as Func

struct VertexInput {
  @location(0) controls: vec4<u32>,
  @location(1) position: vec3<f32>,
  @location(2) color: vec3<f32>,
  @location(3) tex_coord: vec2<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) color: vec3<f32>,
  @location(1) is_click: u32,
};

struct Controls {
  @location(0) size: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> controls: Controls;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  var position: vec3<f32>;

  let is_click = model.controls.x & 1u;
  let converted = (model.controls.x >> 1u) & 1u;

  if bool(converted) {
    position = model.position;
  } else {
    position = Func::to_shader_coord(model.position, controls.size);
  }

  out.color = model.color;
  out.clip_position = vec4<f32>(position, 1.0);
  out.is_click = is_click;
  
  return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  if bool(in.is_click) {
    let b_color = vec4<f32>(in.color, 0.3);
    let s_color = vec4<f32>(0.235, 0.564, 1., 0.);

    return b_color + s_color;
  } else {
    return vec4<f32>(in.color, 1.0);
  }
}
