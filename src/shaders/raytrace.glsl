// #define GBUFFER_BINDGROUP
// #define SSRT_DATA_BINDGROUP 
// #define DEPTH_LOD_BINDGROUP
// #define VP_BINDGROUP
uniform UniformViewProj {
    mat4 view;
    mat4 proj;
}VP;


layout(set=GBUFFER_BINDGROUP, binding=0) uniform texture2D t_depth;
// layout(set=1, binding=0, r32f) readonly uniform image2D t_depth;
layout(set=GBUFFER_BINDGROUP, binding=1,rgba32f)  readonly uniform image2D rt0;
layout(set=GBUFFER_BINDGROUP, binding=2,rgba32f)  readonly uniform image2D rt1;
layout(set=GBUFFER_BINDGROUP, binding=3,rgba32f)  readonly uniform image2D rt2;

layout(set=VP_BINDGROUP, binding=0)
uniform UniformViewProj {
    mat4 view;
    mat4 pro
}VP;

layout(set=SSRT_DATA_BINDGROUP, binding=0Z)
uniform SSRTData {
    int image_width;
    int image_height;
    int lod_width;
    int lod_height;
    int max_level;
    float near;
    vec3 view_dir;
    vec3 eye_pos;
};


layout(set=DEPTH_LOD_BINDGROUP, binding=1)
uniform texture2D depth_lod[];



const float PI = 3.1415926535384;
float rand() {
    seed = (seed * uint(1103515245)) + uint(12345);
    return float(seed) / float(uint(0xFFFFFFFF));
}

float origin()      { return 1.0 / 32.0; }
float float_scale() { return 1.0 / 65536.0; }
float int_scale()   { return 256.0; }

// Normal points outward for rays exiting the surface, else is flipped.
vec3 offset_ray(const vec3 p, const vec3 n)
{
  ivec3 of_i = ivec3(int_scale() * n.x, int_scale() * n.y, int_scale() * n.z);

  vec3 p_i = vec3( 
      intBitsToFloat(floatBitsToInt(p.x)+((p.x < 0.0) ? -of_i.x : of_i.x)),
      intBitsToFloat(floatBitsToInt(p.y)+((p.y < 0.0) ? -of_i.y : of_i.y)),
      intBitsToFloat(floatBitsToInt(p.z)+((p.z < 0.0) ? -of_i.z : of_i.z)));

  return vec3(abs(p.x) < origin() ? p.x+ float_scale()*n.x : p_i.x,
                abs(p.y) < origin() ? p.y+ float_scale()*n.y : p_i.y,
                abs(p.z) < origin() ? p.z+ float_scale()*n.z : p_i.z);
}

vec3 offset_along_normal(const vec3 p, const vec3 n, vec3 dir, float k){
    return p + n * k / dot(dir, n);
}
float get_metallic(ivec2 pixel){
    return imageLoad(rt0, pixel).w;
}
float get_roughness(ivec2 pixel){
    return imageLoad(rt1, pixel).w;
}
vec3 get_color(ivec2 pixel){
    return imageLoad(rt0, pixel).rgb;
}
vec3 get_normal(ivec2 pixel){
    return imageLoad(rt1, pixel).xyz;
}
vec3 get_pos(ivec2 pixel){
    return imageLoad(rt2, pixel).xyz;
}
struct Ray {
    vec3 o;
    vec3 d;
};
struct ScreenSpaceRay {
    vec3 o;
    vec3 d;
    float tmax;
};
struct SSTraceRecord {
    Ray ray;
    vec2 pixel;
    float tmax;
    // float depth;
};
struct HitRecord {
    float t;
    vec2 pixel;
};


