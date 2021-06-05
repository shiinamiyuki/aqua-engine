#version 450

layout(location=0) in vec3 v_normal;
layout(location=1) in vec3 v_pos;

layout(location=0) out vec4 f_normal;
layout(location=1) out vec4 f_pos;

void main(){
    f_normal = vec4(normalize(v_normal), 1.0f);
    f_pos = vec4(v_pos, 1.0f);
}
