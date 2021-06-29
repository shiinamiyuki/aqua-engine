// #define GBUFFER_BINDGROUP
// #define SSRT_DATA_BINDGROUP 
// #define DEPTH_LOD_BINDGROUP
// #define VP_BINDGROUP

layout(set=VP_BINDGROUP, binding=0)
uniform UniformViewProj {
    mat4 view;
    mat4 proj;
}VP;


layout(set=GBUFFER_BINDGROUP, binding=0) uniform texture2D t_depth;
// layout(set=1, binding=0, r32f) readonly uniform image2D t_depth;
layout(set=GBUFFER_BINDGROUP, binding=1,rgba32f)  readonly uniform image2D rt0;
layout(set=GBUFFER_BINDGROUP, binding=2,rgba32f)  readonly uniform image2D rt1;
layout(set=GBUFFER_BINDGROUP, binding=3,rgba32f)  readonly uniform image2D rt2;

struct SSRTData {
    int image_width;
    int image_height;
    int lod_width;
    int lod_height;
    int max_level;
    float near;
    vec3 view_dir;
    vec3 eye_pos;
};
layout(set=SSRT_DATA_BINDGROUP, binding=0)
uniform SSRTDataUniform {
    SSRTData ssrt_data_uniform;
};
SSRTData ssrt;

layout(set=DEPTH_LOD_BINDGROUP, binding=0)
uniform texture2D depth_lod[];
uint seed;


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




struct Frame {
    vec3 T; //x
    vec3 N; //y
    vec3 B; //z
};
Frame frame_from1(vec3 n){
    Frame self;
    vec3 t;
    if(abs(n.x)>abs(n.y)){
        t = normalize(vec3(-n.z, 0, n.x));
    }else{
        t = normalize(vec3(0, n.z, -n.y));
    }
    self.N = n;
    self.T = t;
    self.B = normalize(cross(self.N, self.T));
    return self;
}

Frame ssframe(vec3 w){

/*

N -> T
|
V
B
*/
    Frame self;
    
    self.N = w;
    self.T = normalize(cross(self.N, vec3(0, 1, 0)));
    self.B = normalize(cross(self.N, self.T));
    return self;
}
vec3 to_local(const Frame self, vec3 w){
    return vec3(dot(w, self.T), dot(w, self.N), dot(w, self.B));
}
vec3 to_world(const Frame self, vec3 w){
    return self.T * w.x + self.N * w.y + self.B * w.z;
}

vec3 project_point(mat4 m, vec3 p){
    vec4 h = m * vec4(p, 1.0);
    return h.xyz / h.w;
}
vec3 ndc_to_pixel(vec3 p){
    p.y = -p.y;
    p.xy = ((p.xy + 1.0) / 2.0  * vec2(ssrt.image_width, ssrt.image_height));
    return p;
}
ScreenSpaceRay create_ss_ray(const Ray ray, float tmax){

    vec3 p0 = project_point(VP.view, ray.o);
    vec3 d = normalize(mat3(VP.view) * ray.d);// * tmax;
    // vec3 d = normalize(mat3(VP.view) * ray.d);//assume det(VP.view) = 1
    // {
    //     vec3 p1 = project_point(VP.view, ray.o + ray.d * tmax);
    //     d = p1 - p0;
    //     tmax = length(d);
    //     d = normalize(d);
    // }
    if(d.z > 0.0){
        tmax = min(tmax, abs(p0.z / d.z) * 0.999);   
    }
    vec3 p1 = p0 + tmax * d;
    p0 = ndc_to_pixel(project_point(VP.proj, p0));
    p1 = ndc_to_pixel(project_point(VP.proj, p1));
    ScreenSpaceRay ssray;
    ssray.o = p0;
    d = p1 - p0;
    float len = length(d.xy);
    ssray.d = d / len;
    ssray.tmax = len;

    return ssray;
    
}
// ivec2 ndc_to_int(vec2 p){
//     p.y = -p.y;
//     return ivec2((p + 1.0) / 2.0  * vec2(image_width, image_height));
// }
// vec2 ndc_to_tc(vec2 p){
//     p.y = -p.y;
//     return (p + 1.0) / 2.0;
// }
float get_depth(ivec2 pixel, int level){
    if(level == 0){
        vec2 tc = vec2(pixel) / vec2(ssrt.image_width, ssrt.image_height);
        return  texture(sampler2D(t_depth, s_sampler), tc).x;
    }
    vec2 tc = vec2(pixel) / vec2(ssrt.lod_width, ssrt.lod_height);
    // tc.y = 1.0 - tc.y;
    return texture(sampler2D(depth_lod[level-1], s_sampler), tc).x;
}
bool test_hit(vec3 p, int level){
    ivec2 pixel = ivec2(p.xy);
    if(any(greaterThanEqual(pixel, ivec2(ssrt.image_width, ssrt.image_height))))
        return false;
    if(any(lessThan(pixel, ivec2(0))))
        return false;
    float depth = get_depth(pixel, level);
    if(p.z > depth){
        return true;
    }
    return false;
}
#define SSRT_MIPMAP
#ifdef SSRT_MIPMAP
bool trace(const in SSTraceRecord record, inout HitRecord hit, inout vec3 debug){
    ScreenSpaceRay ray = create_ss_ray(record.ray, record.tmax);
    if(dot(ray.d, ray.d)< 1e-5){
        return false;
    }
    debug = vec3(normalize(vec2(ray.d.xy)), 0.0);
    float march_step_base = 1.01;
    vec3 dir = normalize(ray.d);
    float t = 1.001;
    ivec2 prev_pixel = ivec2(ray.o.xy);
    int level = 0;

    int accum_level0_steps = 0;

    while(t < ray.tmax){
        float march_step = march_step_base;
        for(int i =0;i<level;i++){
            march_step *= 1.6;
        }
        float next_t = t + march_step;
        
        vec3 p = ray.o + ray.d * next_t;
        ivec2 pixel = ivec2(p.xy);
        bool oob = false;
        if(any(greaterThanEqual(pixel, ivec2(ssrt.image_width, ssrt.image_height))))
            oob = true;
        if(any(lessThan(pixel, ivec2(0))))
            oob = true;
        if(next_t > ray.tmax || oob){
            if(level == 0)
                break;
            level--;
            continue;
        }
        if(test_hit(p, level)) {
            if(level == 0){
                hit.t = next_t;
                hit.pixel = p.xy;
                return true;
            }
            level--;
            continue;
        }else{
            t = next_t;
        }
        if(level == 0){
            accum_level0_steps++;
        }else{
            accum_level0_steps = 0;
        }
        if(level == 0){
            if(accum_level0_steps >= 8) {
                level = min(level+1,ssrt.max_level);
            }
        }else{
            level = min(level+1,ssrt.max_level);
        }
    }
    return false;
} 
#else
bool trace(const in SSTraceRecord record, inout HitRecord hit, inout vec3 debug){
    ScreenSpaceRay ray = create_ss_ray(record.ray, record.tmax);
    if(dot(ray.d, ray.d)< 1e-5){
        return false;
    }
    debug = vec3(normalize(vec2(ray.d.xy)), 0.0);
    float pixel_dist = length(ray.d.xy);
    // float march_step_ss = 2.0 / max(image_width, image_height);// / 4.0;// * pixel_dist / 1.0;//1.0 / pixel_dist;
    // float march_step = march_step_ss / length(ray.d.xy) * length(ray.d);
    float march_step = 2.01;
    vec3 dir = normalize(ray.d);
    float t = 1.001;
    ivec2 prev_pixel = ivec2(ray.o.xy);
    while(t < ray.tmax){
        vec3 p = ray.o + ray.d * t;
        ivec2 pixel = ivec2(p.xy);
        // if(all(equal(prev_pixel, pixel))){
        //     t += march_step;
        //     continue;
        // }
        if(any(greaterThanEqual(pixel, ivec2(image_width, image_height))))
            return false;
        if(any(lessThan(pixel, ivec2(0))))
            return false;
        // float depth = texture(sampler2D(t_depth, s_sampler),ndc_to_tc(p.xy)).x;
        float depth = get_depth(pixel, 0);
        if(p.z > depth){
            hit.t = t;
            hit.pixel = p.xy;
            return true;
        }
        t += march_step;
        prev_pixel = pixel;
    }
    return false;
} 
#endif

void init_ssrt() {
    ssrt = ssrt_data_uniform;
}