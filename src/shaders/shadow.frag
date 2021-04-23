#version 450
layout(location=0) in vec3 pos;
layout(location=0) out float depth;
void main(){
    depth = length(pos);
}