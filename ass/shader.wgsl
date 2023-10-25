var<private> QUAD_VERTICES : array<vec4<f32>, 6> = array(
  vec4(-0.5, -0.5, 0., 1.),
  vec4(-0.5, 0.5, 0., 1.),
  vec4(0.5, -0.5, 0., 1.),
  vec4(0.5, 0.5, 0., 1.),
  vec4(0.5, -0.5, 0., 1.),
  vec4(-0.5, 0.5, 0., 1.),
);

var<private> WORLD_TO_PIXEL: mat4x4f = mat4x4(
  32., -16., 0., 0.,
  32., 16., 0., 0.,
  0., 0., 1., 0.,
  0., 0., 0., 1.
);

struct PushConstants {
  screen_size: vec2i,
};

var<push_constant> PUSH_CONSTANTS: PushConstants;

// Vertex shader
struct InstanceData {
  @location(0) position: vec3<f32>,
  @location(1) color: vec4<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32, model: InstanceData) -> VertexOutput {    
    var out: VertexOutput;
    let world_pos = QUAD_VERTICES[in_vertex_index] + vec4(model.position, 0.0f);
    let pixel_pos = WORLD_TO_PIXEL * world_pos;
    let screen_pos = pixel_pos.xy / (vec2f(PUSH_CONSTANTS.screen_size) * 0.5f);
    out.clip_position = vec4<f32>(screen_pos, 0.0f, 1.0f);
    out.color = model.color;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
