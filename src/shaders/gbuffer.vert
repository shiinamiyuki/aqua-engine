#version 450
layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;


/*
GBuffer Layout

rt0: rgb:color, a: metallic
rt1: rgb:normal, b: roughness
rt2: rgb:world pos
*/

layout(location=0) out vec3 v_color;
layout(location=1) out vec3 v_normal;
layout(location=2) out vec3 v_pos;
layout(location=3) out float v_metallic;
layout(location=4) out float v_roughness;


layout(set=0, binding=0)
uniform UniformViewProj {
    mat4 view;
    mat4 proj;
};

void main() {
    mat4 vp = proj * view;
    v_normal = a_normal;
    gl_Position = vp * vec4(a_position, 1.0);
    v_pos = a_position;
    v_color = vec3(1);
    v_metallic = 0.0;
    v_roughness = 0.0;
}
 