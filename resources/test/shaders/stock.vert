#version 450

layout (location = 0) in vec3 a_vertex;
layout (location = 1) in vec3 a_normal;
layout (location = 2) in vec2 a_tex_coord;

layout (set = 0, binding = 0) uniform UniformBufferObject {
    mat4 mvp_matrix;
} ubo;

layout (location = 0) out vec2 v_tex_coord;

void main() {
    v_tex_coord = a_tex_coord;
    gl_Position = ubo.mvp_matrix * vec4(a_vertex, 1.0);
}
