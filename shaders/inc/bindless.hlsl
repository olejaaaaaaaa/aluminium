
struct Camera {
    float4x4 view;           
    float4x4 proj;           
    float4x4 view_proj;      
    float4x4 inv_view;       
    float4x4 inv_proj;       
    float4x4 inv_view_proj;  
};

struct FrameValues {
    uint2   resolution;
    uint    frame_index;
    float   delta_time_sec;
    float   time_sec;
    float3  pad;
};

struct Transform {
    float4   rot;
    float4   scale;
    float4   pos;
};

[[vk::push_constant]]
struct {
    // 4 bytes
    uint transform_index;
    // 4 bytes
    uint prev_transform_index;
    // 4 bytes * 25 = 100 bytes
    uint image_indices[25];
    // 4 bytes * 5 = 20 bytes
    uint image_samplers[5];
    // sum = 4 + 4 + 100 + 20 = 128
} push_constants;

[[vk::binding(0, 0)]] StructuredBuffer<Camera> camera;
[[vk::binding(0, 1)]] StructuredBuffer<FrameValues> value;
[[vk::binding(0, 2)]] StructuredBuffer<Transform> transforms;
[[vk::binding(0, 3)]] Texture2D textures[];
[[vk::binding(0, 4)]] SamplerState samplers[];
