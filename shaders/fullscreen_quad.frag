#version 450
#extension GL_EXT_nonuniform_qualifier : require

layout(location = 0) in vec2 inUV;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform texture2D textures[];
layout(set = 0, binding = 1) uniform sampler tex_sampler;
layout(set = 1, binding = 0) uniform texture2D gbuffer;

layout(push_constant) uniform push {
    uint tex_idx[8];
    float user_data[24];
};

void main() {
    outColor = texture(
        sampler2D(gbuffer, tex_sampler),
        inUV
    );
}