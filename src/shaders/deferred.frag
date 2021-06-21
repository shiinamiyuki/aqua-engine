#version 450

layout(location=0) in vec3 v_pos;
layout(location=0) out vec4 f_color;
layout(set = 0, binding = 0) uniform texture2D t_normal;
layout(set = 0, binding = 1) uniform sampler s_normal;

void main(){
    vec2 tc = v_pos.xy * 0.5 + 0.5;
    tc.y = 1.0 - tc.y;
    vec3 col = texture(sampler2D(t_normal, s_normal), tc).xyz;
    // col = pow(col, vec3(1.0/2.2));
    f_color = vec4(col, 1.0);
}