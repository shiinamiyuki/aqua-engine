#version 450
layout(location=0) in vec3 a_position;
layout(location=0) out vec3 pos;
layout( push_constant ) uniform FaceIndex {
  int face_idx;
};

struct ViewProj {
    mat4 view;
    mat4 proj;
};
layout(set=0,binding=0)
uniform _ViewProj {
    ViewProj vp[6];
};

void main(){
    mat4 view = vp[face_idx].view;
    mat4 proj = vp[face_idx].proj;
    vec4 v = view * vec4(a_position, 1.0);

    vec4 projected = proj * v;
    
    pos = v.xyz / v.w;
    gl_Position = projected;
}