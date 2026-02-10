
struct CameraData {
    float4x4 view;
    float4x4 proj;
    float4   pos;
};

cbuffer CameraBuffer : register(b0) {
    CameraData camera;
};

struct VSInput
{
    float3 position : POSITION;     
    float3 color    : COLOR0;      
};

struct VSOutput
{
    float4 position : SV_POSITION;
    float3 color    : COLOR0;      
};

VSOutput main(VSInput input)
{
    VSOutput output;

    float4 worldPos = float4(input.position, 1.0f);
    output.position = mul(camera.proj, mul(camera.view, worldPos));
    output.color = input.color;

    return output;
}