struct PSInput
{
    float4 position : SV_POSITION;
    float2 uv : TEXCOORD0;
};

float4 main(PSInput input) : SV_TARGET
{
    // Яркий градиент
    return float4(input.uv.x, input.uv.y, 1.0 - input.uv.x, 1.0);
}