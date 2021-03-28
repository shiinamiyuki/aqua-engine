#version 450
layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;

layout(location=0) out vec3 v_normal;

void main() {
    v_normal = normalize(a_normal);
    gl_Position = vec4(a_position, 1.0);
}
 