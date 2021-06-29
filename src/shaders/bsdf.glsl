
struct BSDF {
    Frame frame;
    vec3 n;
    vec3 color;
};

vec2 sample_disk(){
    float r = rand();
    float theta = rand() * 2.0 * PI;
    r = sqrt(r);
    return r * vec2(sin(theta), cos(theta));
}

vec3 sample_hemisphere(){
    vec2 uv = sample_disk();
    float r = dot(uv, uv);
    float h = sqrt(1.0 - r);
    return vec3(uv.x, h, uv.y);
}
vec3 evaluate_bsdf(const BSDF bsdf, vec3 wo, vec3 wi){
    wo = to_local(bsdf.frame, wo);
    wi = to_local(bsdf.frame, wi);
    if(wo.y * wi.y > 0.0){
        return bsdf.color / PI;
    }else{
        return vec3(0);
    }
}
struct BSDFSample {
    vec3 f;
    float pdf;
    vec3 wi;
    
};

BSDFSample sample_bsdf(const BSDF bsdf, vec3 wo){
    vec3 w = sample_hemisphere();
    float pdf = abs(w.y) / PI;
    vec3 f = bsdf.color / PI;
    wo = to_local(bsdf.frame, wo);
    if(wo.y * w.y < 0.0){
        w.y = -w.y;
    }
    w = to_world(bsdf.frame, w);
    return BSDFSample(f, pdf, w);
}