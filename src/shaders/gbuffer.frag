#version 450

layout(location=0) in vec3 v_color;
layout(location=1) in vec3 v_normal;
layout(location=2) in vec3 v_pos;
layout(location=3) in float v_metallic;
layout(location=4) in float v_roughness;


/*
GBuffer Layout

rt0: rgb:color, a: metallic
rt1: rgb:normal, b: roughness
rt2: rgb:world pos
*/


layout(location=0) out vec4 rt0;
layout(location=1) out vec4 rt1;
layout(location=2) out vec4 rt2;

void main(){
    vec3 f_normal = normalize(v_normal);
    vec3 f_pos = v_pos;
    float f_metallic = v_metallic;
    float f_roughness = v_roughness;
    vec3 f_color = v_color;
    rt0 = vec4(f_color, f_metallic);
    rt1 = vec4(f_normal, f_roughness);
    rt2 = vec4(v_pos, 0.0);
}
