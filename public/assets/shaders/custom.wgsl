struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@group(0) @binding(0) 
var<storage, read_write> storage_buffer: array<VertexInput, 10>;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  
  out.color = model.color;
  out.clip_position = vec4<f32>(model.position, 1.0);
  
  var a: f32 = 0.0;

  for (var i: i32 = 0; i < 10; i++) {
    a = a + 0.5;

    let cz = storage_buffer[i].color.z;
    let px = storage_buffer[i].position.x;

    storage_buffer[i].color = vec3<f32>(px, a*2.0, a*3.0);
    storage_buffer[i].position = vec3<f32>(a, a*2.0, cz*3.0);
  }
  
  return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(in.color, 1.0);
}
