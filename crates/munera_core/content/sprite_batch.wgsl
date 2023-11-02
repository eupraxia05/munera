struct PushConstants {
  pixel_pos: vec2i,
  gbuffer_size: vec2u
}

var<push_constant> PUSH_CONSTANTS: PushConstants;

struct VertexOutput {
  @builtin(position) clip_position: vec4f,
}

@vertex
fn vs_main() -> VertexOutput {
  var out: VertexOutput;
  out.clip_position = vec4f(
    vec2f(PUSH_CONSTANTS.pixel_pos) / vec2f(PUSH_CONSTANTS.gbuffer_size) * 2.0f,
    0.0f, 1.0f);
  out.clip_position.y *= -1.0f;
  return out;
}

@fragment
fn fs_main() -> @location(0) vec4f {
  return vec4f(0.0f, 0.0f, 1.0f, 1.0f);
}