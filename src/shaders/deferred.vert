#version 450
layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;

// layout(location=0) out vec3 v_normal;
layout(location=0) out vec3 v_pos;


void main() {
    vec3 p = a_position;
    p.xy *= 2.;
    gl_Position = vec4(p, 1.0);
    v_pos = p;
}
 