#version 440

layout(location = 0) in vec3 vPos;
layout(location = 1) in vec3 vColor;

layout(location = 0) out vec3 fragColor;

layout(set = 0, binding = 0) uniform CameraBuffer {
    mat4 view;
    mat4 proj;
    vec4 pos;
} camera; 

void main() {
    gl_Position = camera.proj * camera.view * vec4(vPos, 1.0);
    fragColor = vColor;
}