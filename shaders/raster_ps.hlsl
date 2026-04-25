[[vk::push_constant]] struct Push {
    // 4 * 8 = 32 bytes
    uint tex_idx[8];    
    // 24 * 4 = 96 bytes
    float user_data[24];       
} push;


struct PSInput
{
    float4 position : SV_POSITION;
    float4 color    : COLOR0;
};

float4 main(PSInput input) : SV_TARGET
{
    return input.color;
}