struct VSOutput
{
    float4 position : SV_POSITION;
    float2 uv : TEXCOORD0;
};

VSOutput main(uint vertexID : SV_VertexID)
{
    VSOutput output;
    
    // Генерируем UV координаты из vertexID
    float2 uv = float2((vertexID << 1) & 2, vertexID & 2);
    output.uv = uv;
    
    // Преобразуем в clip space (-1 до 1)
    output.position = float4(uv * float2(2.0, -2.0) + float2(-1.0, 1.0), 0.0, 1.0);
    
    return output;
}