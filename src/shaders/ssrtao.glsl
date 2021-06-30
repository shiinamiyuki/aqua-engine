vec3 ssao(ivec2 pixel){
    vec3 normal = get_normal(pixel);
    vec3 w = sample_hemisphere();

    Frame frame = frame_from1(normal);
    w = to_world(frame, w);

    Ray ray = Ray(get_pos(pixel), w);
    SSTraceRecord record;
    record.ray = ray;
    record.pixel = pixel;
        // record.depth = 
    record.tmax = 3.0;
    vec3 debug;
    HitRecord hit;
    if(trace(record, hit, debug)){
        pixel = ivec2(hit.pixel);
        return vec3(0);
    }else{
        return vec3(1);
    }
}