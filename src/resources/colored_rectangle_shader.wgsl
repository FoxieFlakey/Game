// Follow closely with src/ui/primitives/model/colored_rectangle.rs
struct Vertex {
  @location(0) pos: vec3f,
  @location(1) transform_0: vec4f,
  @location(2) transform_1: vec4f,
  @location(3) transform_2: vec4f,
  @location(4) transform_3: vec4f,
  @location(5) color: vec4f
}

struct VertexOutput {
  @builtin(position) result_pos: vec4f,
  @location(0) color: vec4f
}

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
  out.result_pos = transform * vec4f(input.pos, 1.0);
  out.color = input.color;
  return out;
}

struct FragmentInput {
  @location(0) color: vec4f
}

@fragment
fn fs_main(
  input: FragmentInput
) -> @location(0) vec4f {
  return input.color;
}


