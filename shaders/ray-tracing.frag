#version 450
precision highp float;

layout(location = 0) out vec4 fragColor;

const float PI = 3.141592653589793;
const int MAX_BOUNCES = 30000000;
const vec3 ambient = vec3(0.03);
const vec3 lightColor = vec3(1.0, 0.9, 0.7);
const float lightRadius = 1.5;
const float lightIntensity = 15.0;

const vec3 cameraPos = vec3(5.0, 2.0, -1.0);
const vec3 cameraTarget = vec3(-1.0, 0.5, 0.0);
const vec3 lightOrigin = vec3(3.0, 5.0, -1.0);
const vec2 resolution = vec2(1280.0, 720.0);
const float time = 0.0;

struct Hit {
    vec3 pos;
    vec3 normal;
    vec3 color;
    float roughness;
    bool isLight;
};

uint seed;
uint pcg_hash(uint seed) {
    seed = seed * 747796405u + 2891336453u;
    uint word = ((seed >> ((seed >> 28u) + 4u)) ^ seed) * 277803737u;
    return (word >> 22u) ^ word;
}

float rand() {
    seed = pcg_hash(seed);
    return float(seed) * (1.0 / 4294967296.0);
}

vec3 randomInUnitSphere() {
    float z = rand() * 2.0 - 1.0;
    float t = rand() * 2.0 * PI;
    float r = sqrt(1.0 - z*z);
    return vec3(r * cos(t), r * sin(t), z);
}

vec2 sphIntersect(vec3 ro, vec3 rd, vec3 ce, float ra) {
    vec3 oc = ro - ce;
    float b = dot(oc, rd);
    float c = dot(oc, oc) - ra*ra;
    float h = b*b - c;
    if(h < 0.0) return vec2(-1.0);
    h = sqrt(h);
    return vec2(-b-h, -b+h);
}

float plaIntersect(vec3 ro, vec3 rd, vec4 p) {
    return -(dot(ro,p.xyz)+p.w)/dot(rd,p.xyz);
}

bool intersect(vec3 ro, vec3 rd, inout Hit hit) {
    const vec3 sphPositions[4] = vec3[4](
        vec3(-0.8, 0.2, -0.5),
        vec3(-0.3, 0.4, 0.7),
        vec3(0.7, 0.3, -0.4),
        vec3(0.5, 0.5, 0.5)
    );
    
    const vec3 sphColors[4] = vec3[4](
        vec3(0.2, 0.6, 1.0),
        vec3(1.0, 0.3, 0.4), 
        vec3(0.4, 1.0, 0.3), 
        vec3(1.0, 0.7, 0.5)
    );
    
    float tmin = 1e38;
    bool hasHit = false;
    
    const vec4 plane = vec4(0.0, 1.0, 0.0, 0.0);
    float t = plaIntersect(ro, rd, plane);
    if(t > 0.0 && t < tmin) {
        tmin = t;
        hit.pos = ro + rd * t;
        hit.normal = vec3(0, 1, 0);
        hit.color = vec3(0.9, 0.8, 0.7) * 0.5;
        hit.roughness = 0.3;
        hit.isLight = false;
        hasHit = true;
    }
    
    for(int i = 0; i < 4; i++) {
        vec2 res = sphIntersect(ro, rd, sphPositions[i], 0.3);
        if(res.x > 0.0 && res.x < tmin) {
            tmin = res.x;
            hit.pos = ro + rd * tmin;
            hit.normal = normalize(hit.pos - sphPositions[i]);
            hit.color = sphColors[i];
            hit.roughness = 0.2;
            hit.isLight = false;
            hasHit = true;
        }
    }
    
    vec2 res = sphIntersect(ro, rd, lightOrigin, lightRadius);
    if(res.x > 0.0 && res.x < tmin) {
        tmin = res.x;
        hit.pos = ro + rd * tmin;
        hit.normal = normalize(hit.pos - lightOrigin);
        hit.color = lightColor;
        hit.roughness = 0.0;
        hit.isLight = true;
        hasHit = true;
    }
    
    return hasHit;
}

vec3 trace(vec3 ro, vec3 rd) {
    vec3 accum = vec3(0.0);
    vec3 mask = vec3(1.0);
    
    for(int bounce = 0; bounce < MAX_BOUNCES; bounce++) {
        Hit hit;
        if(!intersect(ro, rd, hit)) {
            float t = 0.5*(rd.y + 1.0);
            vec3 sky = mix(vec3(0.5, 0.7, 1.0), vec3(0.1, 0.1, 0.3), t);
            accum += mask * sky;
            break;
        }
        
        if(hit.isLight) {
            accum += mask * hit.color;
            break;
        }
        
        vec3 lightDir = normalize(lightOrigin - hit.pos);
        float NdotL = max(dot(hit.normal, lightDir), 0.0);
        float dist = distance(hit.pos, lightOrigin);
        float attenuation = lightIntensity / (dist * dist);
        accum += mask * hit.color * NdotL * attenuation * lightColor;
        
        vec3 randomDir = normalize(hit.normal + randomInUnitSphere());
        rd = normalize(mix(reflect(rd, hit.normal), randomDir, hit.roughness));
        ro = hit.pos + rd * 0.001;
        mask *= hit.color;
    }
    
    return accum;
}

void main() {

    seed = uint(gl_FragCoord.x) * 1973u + uint(gl_FragCoord.y) * 9277u;
    
    vec2 uv = (2.0 * gl_FragCoord.xy - resolution) / min(resolution.x, resolution.y);
    uv.y *= -1.0;
    
    vec3 w = normalize(cameraTarget - cameraPos);
    vec3 u = normalize(cross(w, vec3(0.0, 1.0, 0.0)));
    vec3 v = cross(u, w);
    vec3 rd = normalize(uv.x * u + uv.y * v + 1.5 * w);
    
    vec3 color = trace(cameraPos, rd);
    
    color = pow(color, vec3(1.0/2.2));
    fragColor = vec4(color, 1.0);
}