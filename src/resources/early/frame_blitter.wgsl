// Follow closely with src/rendering/translator.rs
struct Vertex {
  @location(0) pos: vec3f,
  @location(1) tex_coord: vec2f
}

struct VertexOutput {
  @builtin(position) result_pos: vec4f,
  @location(0) tex_coord: vec2f
}

@vertex
fn vs_main(
  input: Vertex,
  @builtin(instance_index) index: u32
) -> VertexOutput {
  var out: VertexOutput;
  out.result_pos = vec4(input.pos, 1.0);
  out.tex_coord = input.tex_coord;
  return out;
}

@group(0) @binding(0)
var frame_sampler: sampler;

@group(0) @binding(1)
var input_frame: texture_2d<f32>;

struct FragmentInput {
  @location(0) tex_coord: vec2f
}

@fragment
fn fs_main(
  input: FragmentInput
) -> @location(0) vec4f {
  return textureSample(input_frame, frame_sampler, input.tex_coord);
}


