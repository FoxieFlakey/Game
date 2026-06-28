// Follow closely with src/rendering/translator.rs
struct Vertex {
  @location(0) pos: vec3f,
  @location(1) tex_coord: vec2f
}

struct VertexOutput {
  @builtin(position) result_pos: vec4f,
  @location(0) tex_coord: vec2f
}

struct Uniforms {
  // The width and height of the output
  // in window
  output_width: f32,
  output_height: f32,
  
  // The width and height of game
  // viewport
  render_width: f32,
  render_height: f32,
}

@group(0) @binding(2) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(
  input: Vertex,
  @builtin(instance_index) index: u32
) -> VertexOutput {
  var out: VertexOutput;
  out.result_pos = vec4(input.pos, 1.0);
  out.tex_coord = input.tex_coord;
  
  let output_aspect = uniforms.output_width / uniforms.output_height;
  let render_aspect = uniforms.render_width / uniforms.render_height;

  if output_aspect > 1.0 {
    // Output is wider than render output
    // shrink the width of render output,
    // so its not stretch
    
    // The scaling factor is basically at what percentage does height
    // cover the width. didnt touch the height of render output :3
    // so its filled entire height
    out.result_pos.x *= uniforms.output_height / uniforms.output_width;
  } else {
    // Output is taller than render output
    // shrink the height of render output,
    // so its not stretch
    
    // Same logic like before but swapped width and height
    out.result_pos.y *= uniforms.output_width / uniforms.output_height;
  }

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


