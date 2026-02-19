struct Camera {
    float4x4 view;
    float4x4 proj;
    float4   pos;
};

struct Transform {
    float4   rot;
    float4   scale;
    float4   pos;
};

struct FrameValues {
    uint2   resolution;
    uint    frame_index;
    float   delta_time_sec;
    float   time_sec;
    float3  pad;
};

[[vk::binding(0, 0)]] StructuredBuffer<Camera> camera;
[[vk::binding(1, 0)]] ConstantBuffer<FrameValues> frame_values;
[[vk::binding(2, 0)]] StructuredBuffer<Transform> transforms;

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
    output.color = float4(input.color, 1.0) * sin(frame_values.time_sec);

    return output;
}