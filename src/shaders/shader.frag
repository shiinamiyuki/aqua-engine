#version 450

layout(location=0) in vec3 v_normal;
layout(location=1) in vec3 v_pos;
layout(location=0) out vec4 f_color;
layout(set=0, binding=0)
uniform UniformViewProj {
    mat4 view;
    mat4 proj;
};
struct PointLight {
    vec3 position;
    vec3 emission;
};
layout(set=1, binding=0)
uniform _PointLight {
    PointLight point_light;
};

void main() {
    vec3 ns = normalize(v_normal);
    // f_color = vec4(v_normal*0.5+0.5, 1.0);
    vec3 wi = (point_light.position - v_pos);
    float dist = length(wi);
    wi /= dist;
    vec3 color = max(0.0, dot(ns, wi)) * point_light.emission / (dist * dist);
    f_color = vec4(color, 1.0);
}
 