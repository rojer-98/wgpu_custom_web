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
  @location(1) is_click: vec4<u32>,
};

struct Controls {
  @location(0) size: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> controls: Controls;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
  var out: VertexOutput;

  out.color = model.color;
  out.clip_position = vec4<f32>(Func::to_shader_coord(model.position, controls.size), 1.0);
  out.is_click = model.controls;
  
  return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  if in.is_click.x == 1 {
    let b_color = vec4<f32>(in.color, 0.3);
    let s_color = vec4<f32>(0.235, 0.564, 1., 0.);

    return b_color + s_color;
  } else {
    return vec4<f32>(in.color, 1.0);
  }
}
