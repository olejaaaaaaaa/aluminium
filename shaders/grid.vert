#version 450

struct Camera {
    mat4 view;
    mat4 proj;
    vec4 pos;
};

layout(set = 0, binding = 0) uniform CameraBuffer {
    Camera camera;
};

layout(location = 0) out vec3 worldPos;
layout(location = 1) out vec3 nearPoint;
layout(location = 2) out vec3 farPoint;

vec3 gridPlane[6] = vec3[](

    vec3(-1,-1, 0), 
    vec3( 1,-1, 0), 
    vec3(-1, 1, 0), 

    vec3( 1,-1, 0), 
    vec3(-1, 1, 0), 
    vec3( 1, 1, 0)
    
);

vec3 unprojectPoint(float x, float y, float z, mat4 viewProjInv) {
    vec4 unprojectedPoint = viewProjInv * vec4(x, y, z, 1.0);
    return unprojectedPoint.xyz / unprojectedPoint.w;
}

void main() {

    vec3 p = gridPlane[gl_VertexIndex];
    
    mat4 viewProjInv = inverse(camera.proj * camera.view);
    
    nearPoint = unprojectPoint(p.x, p.y, 0.0, viewProjInv);
    farPoint = unprojectPoint(p.x, p.y, 1.0, viewProjInv);
    
    gl_Position = vec4(p, 1.0);
}


