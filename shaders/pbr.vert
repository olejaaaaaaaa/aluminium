#version 450

layout(location = 0) in vec4 vPos;
layout(location = 1) in vec4 vNormal;
layout(location = 2) in vec2 vUV;
layout(location = 3) in vec4 vColor;
layout(location = 4) in vec4 vTangent;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragUV;
layout(location = 2) out vec3 fragNormal;
layout(location = 3) out vec3 fragWorldPos;
layout(location = 4) out vec4 fragTangent;

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

void main() {
    gl_Position = ubo.proj * ubo.view * ubo.model * vec4(vPos.xyz, 1.0);
    vec4 worldPos = ubo.model * vec4(vPos.xyz, 1.0);
    fragWorldPos = worldPos.xyz;
    fragColor = vColor.rgb;
    fragUV = vUV;
    fragNormal = mat3(transpose(inverse(ubo.model))) * vNormal.xyz;
    fragTangent = vTangent;
}