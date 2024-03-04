#define_import_path simple_functions

fn to_shader_coord(model_position: vec3<f32>, size: vec4<f32>) -> vec3<f32> {
  var position: vec3<f32>;
  
  let half_w = size.x / 2.; 
  let half_h = size.y / 2.; 
  let coord = model_position; 

  if coord.x > half_w {
    position.x = coord.x / half_w - 1.; 
  } else {
    position.x = -(1. - coord.x / half_w);
  }

  if coord.y > half_h {
    position.y = -(coord.y / half_h - 1.);
  } else {
    position.y = 1. - coord.y /half_h; 
  }

  return position;
}
