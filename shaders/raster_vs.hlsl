struct Transform {
    float4x4 mvp;
};

[[vk::binding(0, 1)]] StructuredBuffer<Transform> transforms;

[[vk::push_constant]] struct Push {
    uint tex_idx[8];    
    float user_data[24];       
} push;

struct VSInput {
    float4 position : POSITION;  
    float4 normal   : NORMAL; 
    float2 uv       : TEXCOORD0;
    float4 color    : COLOR0;  
    float4 tangent  : TANGENT;
};

struct VSOutput {
    float4 position : SV_POSITION;
    float3 normal   : TEXCOORD0;   // float3 вместо float4
    float3 tangent  : TEXCOORD1;   // float3 вместо float4
    float2 uv       : TEXCOORD2;
};

VSOutput main(VSInput input) {
    VSOutput output;
    Transform t = transforms[(uint)push.user_data[1]];
    output.position = mul(t.mvp, float4(input.position.xyz, 1.0));
    output.uv      = input.uv;
    output.normal  = input.normal.xyz;   // берём xyz
    output.tangent = input.tangent.xyz;
    return output;
}