#version 460 core

uniform vec2 u_screen_size;
in vec2 a_pos;
in vec2 a_tex_coord;
in vec4 a_tint;
out vec4 v_tint;
out vec2 v_tex_coord;

void main() {
    gl_Position = vec4(
        2.0 * a_pos.x / u_screen_size.x - 1.0,
        1.0 - 2.0 * a_pos.y / u_screen_size.y,
        0.0,
        1.0);
    v_tint = a_tint / 255.0;
    v_tex_coord = a_tex_coord;
}