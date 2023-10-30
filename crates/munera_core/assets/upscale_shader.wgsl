var<private> QUAD_VERTICES : array<vec4<f32>, 6> = array(
  vec4(-1., -1., 0., 1.),
  vec4(-1., 1., 0., 1.),
  vec4(1., -1., 0., 1.),
  vec4(1., 1., 0., 1.),
  vec4(1., -1., 0., 1.),
  vec4(-1., 1., 0., 1.),
);

@group(0) @binding(0)
var t_gbuffer: texture_2d<f32>;

@group(0) @binding(1)
var s_gbuffer: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {    
    var out: VertexOutput;
    let pos = QUAD_VERTICES[in_vertex_index];
    out.clip_position = pos;
    out.uv = pos.xy * 0.5 + 0.5;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_gbuffer, s_gbuffer, in.uv);
}
