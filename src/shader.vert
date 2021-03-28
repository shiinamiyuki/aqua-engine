#version 450
layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;

layout(location=0) out vec3 v_normal;

layout(set=0, binding=0)
uniform UniformViewProj {
    mat4 view;
    mat4 proj;
};

void main() {
    mat4 vp = proj * view;
    v_normal = normalize(transpose(inverse(mat3(vp))) * a_normal);
    gl_Position = vp * vec4(a_position, 1.0);
}
 