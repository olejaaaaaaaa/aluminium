#version 450

layout(location = 0) in vec2 inUV;
layout(location = 0) out vec4 outColor;

layout(set = 1, binding = 0) uniform sampler2D frame1;
layout(set = 1, binding = 1) uniform sampler2D frame2;
layout(set = 1, binding = 2) uniform sampler2D frame3;

void main() {

    vec2 uiUV = vec2(inUV.x, 1.0 - inUV.y);
    outColor = texture(frame2, inUV) + texture(frame3, uiUV);

    vec2 pipStart = vec2(0.7, 0.05);
    vec2 pipEnd = vec2(0.95, 0.35);
    
    if (inUV.x >= pipStart.x && inUV.x <= pipEnd.x &&
        inUV.y >= pipStart.y && inUV.y <= pipEnd.y) {

        vec2 pipUV = (inUV - pipStart) / (pipEnd - pipStart);
        outColor = texture(frame1, pipUV);

        float borderWidth = 0.005;
        if (inUV.x < pipStart.x + borderWidth || 
            inUV.x > pipEnd.x - borderWidth ||
            inUV.y < pipStart.y + borderWidth ||
            inUV.y > pipEnd.y - borderWidth) {
            outColor = vec4(1.0, 1.0, 1.0, 1.0);
        }
    }
}