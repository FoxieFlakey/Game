// Follow closely with src/screen/loading_screen/resources.rs
struct Vertex {
  @location(0) pos: vec3f,
  @location(1) tex_coord: vec2f,
  @location(2) transform_0: vec4f,
  @location(3) transform_1: vec4f,
  @location(4) transform_2: vec4f,
  @location(5) transform_3: vec4f
}

struct VertexOutput {
  @builtin(position) result_pos: vec4f,
  @location(0) tex_coord: vec2f
}

struct CameraUniform {
  projection_matrix: mat4x4f
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;

// Vertex shader, does pretty basic operation of
// multiplying vector with transform matrix
@vertex
fn vs_main(
  input: Vertex,
  @builtin(instance_index) index: u32
) -> VertexOutput {
  let transform = mat4x4f(
    input.transform_0,
    input.transform_1,
    input.transform_2,
    input.transform_3
  );
  
  var out: VertexOutput;
  out.result_pos = camera.projection_matrix * transform * vec4f(input.pos, 1.0);
  out.tex_coord = input.tex_coord;
  return out;
}

struct FragmentInput {
  @location(0) tex_coord: vec2f
}

@group(1) @binding(0)
var common_sampler: sampler;
@group(1) @binding(1)
var loading_paw_texture: texture_2d<f32>;

@fragment
fn fs_main(
  input: FragmentInput
) -> @location(0) vec4f {
  return textureSample(loading_paw_texture, common_sampler, input.tex_coord);
}


