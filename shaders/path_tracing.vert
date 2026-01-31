#version 440

layout(location = 0) out vec2 fragUV;

void main() {
    
    vec2 positions[3] = vec2[](
        vec2(-1, -1),
        vec2(3, -1),
        vec2(-1, 3)
    );
    
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
    fragUV = gl_Position.xy * 0.5 + 0.5;
}