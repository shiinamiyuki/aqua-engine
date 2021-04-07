#version 450

layout(location=0) in vec3 v_normal;
layout(location=1) in vec3 v_pos;

layout(location=0) out vec3 f_normal;
layout(location=1) out vec3 f_pos;

void main(){
    f_normal = normalize(v_normal);
    f_pos = v_pos;
}