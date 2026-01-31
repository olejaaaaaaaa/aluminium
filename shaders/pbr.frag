#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragUV;
layout(location = 2) in vec3 fragNormal;
layout(location = 3) in vec3 fragWorldPos;
layout(location = 4) in vec4 fragTangent;

layout(location = 0) out vec4 outColor;

layout(binding = 4) uniform sampler2D albedo_map;           // diffuse/albedo
layout(binding = 2) uniform sampler2D normal_map;           // normal
layout(binding = 1) uniform sampler2D metallic_roughness;   // metallic (B), roughness (G)
layout(binding = 3) uniform sampler2D occlusion_map;        // ao

const vec3 LIGHT_POSITIONS[4] = vec3[](
    vec3(10.0, 10.0, 10.0),
    vec3(-10.0, 10.0, 10.0),
    vec3(10.0, -10.0, 10.0),
    vec3(-10.0, -10.0, 10.0)
);

const vec3 LIGHT_COLORS[4] = vec3[](
    vec3(300.0, 300.0, 300.0),
    vec3(300.0, 300.0, 300.0),
    vec3(300.0, 300.0, 300.0),
    vec3(300.0, 300.0, 300.0)
);

const vec3 CAM_POS = vec3(0.0, 0.0, 5.0);
const float PI = 3.14159265359;

vec3 getNormalFromMap() {
    vec3 N = normalize(fragNormal);
    vec3 T = normalize(fragTangent.xyz);
    vec3 B = cross(N, T) * fragTangent.w;

    mat3 TBN = mat3(T, B, N);

    vec3 tangentNormal = texture(normal_map, fragUV).rgb * 2.0 - 1.0;
    return normalize(TBN * tangentNormal);
}

float DistributionGGX(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;

    float nom   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / max(denom, 0.0001);
}

float GeometrySchlickGGX(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;

    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / max(denom, 0.0001);
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

vec3 fresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

void main() {

    vec3 albedo = texture(albedo_map, fragUV).rgb;
    vec4 mr_sample = texture(metallic_roughness, fragUV);
    float metallic = mr_sample.b;
    float roughness = mr_sample.g;
    float ao = texture(occlusion_map, fragUV).r;

    // Тоновая коррекция (sRGB to linear)
    albedo = pow(albedo, vec3(2.2));
    vec3 N = getNormalFromMap();
    vec3 V = normalize(CAM_POS - fragWorldPos);

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);

    // reflectance equation
    vec3 Lo = vec3(0.0);
    for(int i = 0; i < 10; ++i)
    {
        // calculate per-light radiance
        vec3 L = normalize(LIGHT_POSITIONS[i] - fragWorldPos);
        vec3 H = normalize(V + L);
        float distance = length(LIGHT_POSITIONS[i] - fragWorldPos);
        float attenuation = 1.0 / (distance * distance);
        vec3 radiance = LIGHT_COLORS[i] * attenuation;

        // Cook-Torrance BRDF
        float NDF = DistributionGGX(N, H, roughness);
        float G   = GeometrySmith(N, V, L, roughness);
        vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);

        vec3 numerator    = NDF * G * F;
        float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
        vec3 specular = numerator / denominator;

        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metallic;

        float NdotL = max(dot(N, L), 0.0);
        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }

    vec3 ambient = vec3(0.003) * albedo * ao;
    vec3 color = ambient + Lo;
    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0 / 2.2));
    outColor = vec4(albedo, 1.0);
}