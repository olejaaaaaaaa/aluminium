
struct Transform {
    float4x4   mvp;
};

[[vk::binding(0, 0)]] StructuredBuffer<Transform> transforms;

[[vk::push_constant]] struct Push {
    // 4 * 8 = 32 bytes
    uint tex_idx[8];    
    // 24 * 4 = 96 bytes
    float user_data[24];       
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

    Transform t = transforms[(uint)push.user_data[1]];
    output.position = mul(t.mvp, float4(input.position.xyz, 1.0));

    float depth = input.position.z;
    output.color = float4(depth, depth, depth, 1.0) * input.normal;

    return output;
}