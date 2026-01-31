
layout(set = 0, binding = 0) uniform sampler2D textures[512];
layout(set = 0, binding = 1, rgba8) uniform image2D storageImages[128];

struct TransformData {
    vec4 scale;
    vec4 rot;
    vec4 pos;
};

layout(set = 0, binding = 2) uniform Transforms {
    TransformData data[8192];
} transforms;

struct AABBData {
    vec3 max;
    vec3 min;
} aaabb_data;

layout(set = 0, binding = 3) uniform GlobalCamera {
    mat4 model;
    mat4 view;
    mat4 proj;
} camera;

layout(set = 0, binding = 4) readonly buffer AABB {
    AABBData data[8192];
} aabb;

struct AABBList {
    uint count;
    uint indices[8192];
} aabb_list_data;

layout(set = 0, binding = 5) uniform AABB_LIST {
    AABBData list;
} aabb_list;

layout(push_constant) uniform PushConstants {
    uint transform_id;
    uint texture_ids[16];
} pc;

// sampler2D getTextureByID(uint id) {
//     return textures[texture_ids[id]];
// }

mat4 getGlobalCamera() {
    return camera.proj * camera.view * camera.model;
}

mat4 getCameraView() {
    return camera.view;
}

TransformData getTransform() {
    return transforms.data[pc.transform_id];
}

void getAABB() {

}