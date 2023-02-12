#version 450

layout (location = 0) in vec2 v_tex_coord;

layout (set = 0, binding = 1) uniform sampler2D s_texture;

layout (location = 0) out vec4 o_color;

void main() {
    o_color = texture(s_texture, v_tex_coord);
}
