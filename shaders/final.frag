#version 450

layout(location = 0) in vec2 inUV;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform texture2D frameTex;
layout(set = 0, binding = 1) uniform sampler frameSampler;

void main() {
    outColor = texture(frame123, inUV);
}