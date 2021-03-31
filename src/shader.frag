#version 450

layout(location=0) in vec3 v_color;
layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D u_Textures[2147483647];

void main() {
    f_color = vec4(v_color*0.5+0.5, 1.0);
}
 