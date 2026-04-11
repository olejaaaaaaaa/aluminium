
// struct Camera {
//     float4x4 view;           
//     float4x4 proj;           
//     float4x4 view_proj;      
//     float4x4 inv_view;       
//     float4x4 inv_proj;       
//     float4x4 inv_view_proj;
// };

// struct FrameValues {
//     uint2   resolution;
//     uint    frame_idx;
//     float   delta_time_sec;
//     float   time_sec;
//     float   pad;
// };

// struct Transform {
//     float4   rot;
//     float4   scale;
//     float4   pos;
// };

// [[vk::binding(0, 0)]] Texture2D<float4>           textures[];
// [[vk::binding(1, 0)]] SamplerState                samplers[5];

// [[vk::binding(0, 1)]] ConstantBuffer<Camera>      camera;
// [[vk::binding(1, 1)]] ConstantBuffer<FrameValues> frame_values;
// [[vk::binding(2, 1)]] StructuredBuffer<Transform> transforms;

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
    // 24 * 4 = 92 bytes
    float user_data[23];       
} push;

struct VSInput
{
    float4 position : POSITION;  
    float4 normal   : NORMAL; 
    float2 uv       : TEXCOORD0;
    float4 color    : COLOR0;  
    float4 tangent  : TANGENT;
};

struct VSOutput
{
    float4 position : SV_POSITION;
    float4 color    : COLOR0;      
};

VSOutput main(VSInput input)
{
    VSOutput output;

    output.position = float4(input.position.xyz * 2.0 * (0.5 + abs(sin(push.user_data[0]))), 1.0);
    output.color = input.tangent * input.color * input.normal;

    return output;
}