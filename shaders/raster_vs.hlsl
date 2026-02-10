
struct CameraData {
    float4x4 view;
    float4x4 proj;
    float4   pos;
};

struct Transform {
    float4x4 rot;
    float4   scale;
    float4   pos;
};

[[vk::binding(0, 0)]] StructuredBuffer<CameraData> camera;
[[vk::binding(0, 1)]] StructuredBuffer<Transform> transforms;

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

    float4 worldPos = float4(input.position, 1.0f);
    output.position = worldPos;
    output.color = float4(input.color, 1.0);

    return output;
}