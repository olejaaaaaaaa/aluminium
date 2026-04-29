static const float PI = 3.14159265359;

[[vk::binding(0, 0)]] Texture2D Textures[];
[[vk::binding(1, 0)]] SamplerState tex_sampler;

[[vk::push_constant]] struct Push {
    uint tex_idx[8];    
    float user_data[24];       
} push;

struct VSOutput {
    float4 position : SV_POSITION;
    float3 normal   : TEXCOORD0;   // float3 вместо float4
    float3 tangent  : TEXCOORD1;   // float3 вместо float4
    float2 uv       : TEXCOORD2;
};

float DistributionGGX(float3 N, float3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float denom = (NdotH * NdotH * (a2 - 1.0) + 1.0);
    return a2 / (PI * denom * denom);
}

float GeometrySchlick(float NdotV, float roughness) {
    float r = roughness + 1.0;
    float k = (r * r) / 8.0;
    return NdotV / (NdotV * (1.0 - k) + k);
}

float GeometrySmith(float3 N, float3 V, float3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    return GeometrySchlick(NdotV, roughness) * GeometrySchlick(NdotL, roughness);
}

float3 FresnelSchlick(float cosTheta, float3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

float4 main(VSOutput input) : SV_TARGET {
    float4 albedo_srgb = Textures[push.tex_idx[0]].Sample(tex_sampler, input.uv);
    float4 metal_rough = Textures[push.tex_idx[1]].Sample(tex_sampler, input.uv);
    float  ao          = Textures[push.tex_idx[2]].Sample(tex_sampler, input.uv).r;
    float3 normal_ts = (Textures[push.tex_idx[3]].Sample(tex_sampler, input.uv).xyz * 2.0) - 1.0;

    float3 N = normalize(input.normal.xyz);
    float3 T = normalize(input.tangent);
    T = normalize(T - dot(T, N) * N); // re-orthogonalize
    float3 B = cross(N, T);
    float3x3 TBN = float3x3(T, B, N);

    // sRGB -> linear
    float3 albedo   = pow(albedo_srgb.rgb, 2.2);
    float metallic  = metal_rough.b;
    float roughness = clamp(metal_rough.g, 0.05, 1.0);

    float3 L = normalize(float3(1.0, 2.0, 1.0));  // directional light direction
    float3 V = normalize(float3(0.0, 0.0, 1.0));  // фиксированный view
    float3 H = normalize(V + L);

    float3 light_color = float3(3.0, 3.0, 3.0);

    float3 F0 = lerp(float3(0.04, 0.04, 0.04), albedo, metallic);

    float  D = DistributionGGX(N, H, roughness);
    float  G = GeometrySmith(N, V, L, roughness);
    float3 F = FresnelSchlick(max(dot(H, V), 0.0), F0);

    float3 specular = (D * G * F) / (4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001);

    float3 kD = (1.0 - F) * (1.0 - metallic);

    float NdotL = max(dot(N, L), 0.0);
    float3 Lo = (kD * albedo / PI + specular) * light_color * NdotL;

    float3 ambient = float3(0.03, 0.03, 0.03) * albedo * ao;
    float3 color   = ambient + Lo;

    // Reinhard tone mapping + gamma
    color = color / (color + 1.0);
    color = pow(color, 1.0 / 2.2);

    return float4(color, albedo_srgb.a);
}