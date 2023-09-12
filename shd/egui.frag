#version 460 core

uniform sampler2D u_sampler;
in vec4 v_tint;
in vec2 v_tex_coord;
out vec4 f_color;

void main() {
    vec4 col = texture2D(u_sampler, v_tex_coord);
    f_color = col * v_tint;
}