#version 450

layout(location = 0) in vec3 nearPoint;
layout(location = 1) in vec3 farPoint;

layout(location = 0) out vec4 outColor;

struct Camera {
    mat4 view;
    mat4 proj;
    vec4 pos;
};

layout(set = 0, binding = 0) uniform CameraBuffer {
    Camera camera;
};

float computeDepth(vec3 pos) {

    mat4 viewProj = camera.proj * camera.view;
    vec4 clipSpacePos = viewProj * vec4(pos, 1.0);
    
    float ndcDepth = clipSpacePos.z / clipSpacePos.w;
    
    return ndcDepth;
}

float computeLinearDepth(vec3 pos) {
    mat4 viewProj = camera.proj * camera.view;
    vec4 clipSpacePos = viewProj * vec4(pos, 1.0);
    float clipSpaceDepth = clipSpacePos.z / clipSpacePos.w;
    
    float near = 0.1;
    float far = 1000.0;
    
    float linearDepth = (2.0 * near * far) / (far + near - clipSpaceDepth * (far - near));
    return linearDepth / far;
}


vec4 grid(vec3 fragPos3D, float scale, bool drawAxis) {

    vec2 coord = fragPos3D.xz * scale;
    vec2 derivative = fwidth(coord);

    vec2 grid = abs(fract(coord - 0.5) - 0.5) / derivative;

    float line = min(grid.x, grid.y);

    float minimumz = min(derivative.y, 1.0);
    float minimumx = min(derivative.x, 1.0);
    
    vec4 color = vec4(0.2, 0.2, 0.2, 1.0 - min(line, 1.0));

    if (drawAxis) {
        float axisWidth = min(minimumx, minimumz);

        if (abs(fragPos3D.x) < axisWidth) {
            color = vec4(0.0, 1.0, 0.0, 1.0);
        }

        else if (abs(fragPos3D.z) < axisWidth) {
            color = vec4(0.0, 0.0, 1.0, 1.0);
        }
    }
    
    return color;
}

void main() {

    float t = -nearPoint.y / (farPoint.y - nearPoint.y);
    
    if (t < 0.0) {
        discard;
    }
    
    vec3 fragPos3D = nearPoint + t * (farPoint - nearPoint);
    
    gl_FragDepth = computeDepth(fragPos3D);
    
    float linearDepth = computeLinearDepth(fragPos3D);
    float fading = max(0.0, 1.0 - linearDepth * 2.0);
    
    vec4 gridColor = grid(fragPos3D, 1.0, true);
    
    vec4 largeGrid = grid(fragPos3D, 0.1, false);
    largeGrid.rgb = vec3(0.5, 0.5, 0.5);
    
    gridColor = mix(gridColor, largeGrid, largeGrid.a * 0.3);
    
    gridColor.a *= fading;
    
    if (gridColor.a < 0.01) {
        discard;
    }
    
    outColor = gridColor;
}



