#version 450

layout(location = 0) out vec2 vUV;

layout(push_constant, std430) uniform push {
    uint tex_idx[8];
    float user_data[24];
};

void main() {

    vec2 positions[3] = vec2[](
        vec2(-1.0, -1.0),
        vec2( 3.0, -1.0),
        vec2(-1.0,  3.0) 
    );

    vec2 uvs[3] = vec2[](
        vec2(0.0, 1.0),
        vec2(2.0, 1.0),
        vec2(0.0, -1.0)
    );
    
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
    vUV = uvs[gl_VertexIndex];
}