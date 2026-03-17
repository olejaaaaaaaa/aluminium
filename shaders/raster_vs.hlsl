
struct Camera {
    float4x4 view;           
    float4x4 proj;           
    float4x4 view_proj;      
    float4x4 inv_view;       
    float4x4 inv_proj;       
    float4x4 inv_view_proj;
    float4   pos;  
};

struct FrameValues {
    uint2   resolution;
    uint    frame_idx;
    float   delta_time_sec;
    float   time_sec;
    float   pad;
};

struct Transform {
    float4   rot;
    float4   scale;
    float4   pos;
};

[[vk::binding(0,  0)]] ConstantBuffer<Camera>      camera;
[[vk::binding(1,  0)]] ConstantBuffer<FrameValues> frame_values;
[[vk::binding(2,  0)]] StructuredBuffer<Transform> transforms;
[[vk::binding(3,  0)]] StructuredBuffer<Transform> prev_transforms;
[[vk::binding(4,  0)]] Texture2D                   textures[];
[[vk::binding(5, 0)]] RWTexture2D                  rw_textures[];
[[vk::binding(6, 0)]] SamplerState                 samplers[5];

const uint SAMPLER_REPEAT = 0;
const uint SAMPLER_CLAMP = 1;
const uint SAMPLER_BORDER = 2;
const uint SAMPLER_MIP_LINEAR = 3;
const uint SAMPLER_MIP_POINT = 4;

[[vk::push_constant]] struct Push {
    // 4 bytes
    uint transform_idx;  
    // 4 * 8 = 32 bytes
    uint tex_idx[8];    
    // 4 * 7 = 28 bytes
    uint rw_tex_idx[7];  
    // 4 * 16 = 64 bytes
    uint user_data[16];       
} push;

struct VSInput
{
    float3 position : POSITION;     
    float3 color    : COLOR0;   
};

struct VSOutput
{
    float4 position : SV_POSITION;
    float4 color    : COLOR0;      
};

VSOutput main(VSInput input)
{
    VSOutput output;

    output.position = float4(input.position, 1.0);
    output.color = float4(input.color, 1.0);

    return output;
}