#version 440

layout(location = 0) in vec2 fragUV;
layout(location = 0) out vec4 outColor;

// ===== СТРУКТУРЫ =====
struct Ray {
    vec3 origin;
    vec3 direction;
};

struct HitInfo {
    bool hit;
    float t;
    vec3 position;
    vec3 normal;
    vec3 albedo;
    vec3 emission;
    bool isEmissive;
};

// ===== RANDOM NUMBER GENERATOR =====
uint rngState;

uint wang_hash(uint seed) {
    seed = (seed ^ 61u) ^ (seed >> 16u);
    seed *= 9u;
    seed = seed ^ (seed >> 4u);
    seed *= 0x27d4eb2du;
    seed = seed ^ (seed >> 15u);
    return seed;
}

float randomFloat() {
    rngState = wang_hash(rngState);
    return float(rngState) / 4294967296.0;
}

vec3 randomInUnitSphere() {
    float z = randomFloat() * 2.0 - 1.0;
    float a = randomFloat() * 6.28318530718;
    float r = sqrt(1.0 - z * z);
    float x = r * cos(a);
    float y = r * sin(a);
    return vec3(x, y, z);
}

vec3 randomHemisphere(vec3 normal) {
    vec3 inUnitSphere = randomInUnitSphere();
    if (dot(inUnitSphere, normal) > 0.0) {
        return inUnitSphere;
    } else {
        return -inUnitSphere;
    }
}

// ===== ПЕРЕСЕЧЕНИЕ СО СФЕРОЙ =====
bool intersectSphere(Ray ray, vec3 center, float radius, out float t) {
    vec3 oc = ray.origin - center;
    float a = dot(ray.direction, ray.direction);
    float b = 2.0 * dot(oc, ray.direction);
    float c = dot(oc, oc) - radius * radius;
    float discriminant = b * b - 4.0 * a * c;
    
    if (discriminant < 0.0) {
        return false;
    }
    
    float sqrtd = sqrt(discriminant);
    float t0 = (-b - sqrtd) / (2.0 * a);
    float t1 = (-b + sqrtd) / (2.0 * a);
    
    if (t0 > 0.001) {
        t = t0;
        return true;
    }
    if (t1 > 0.001) {
        t = t1;
        return true;
    }
    
    return false;
}

// ===== ПЕРЕСЕЧЕНИЕ С ПЛОСКОСТЬЮ (ПОЛ) =====
bool intersectPlane(Ray ray, float y, out float t) {
    if (abs(ray.direction.y) < 0.001) {
        return false;
    }
    
    t = (y - ray.origin.y) / ray.direction.y;
    return t > 0.001;
}

// ===== ТРАССИРОВКА СЦЕНЫ =====
HitInfo traceScene(Ray ray) {
    HitInfo closest;
    closest.hit = false;
    closest.t = 1e10;

    float t;

    // Сфера 1: Матовая (диффузная) - красная
    if (intersectSphere(ray, vec3(-1.2, 1.0, -3.0), 0.9, t)) {
        if (t < closest.t) {
            closest.hit = true;
            closest.t = t;
            closest.position = ray.origin + ray.direction * t;
            closest.normal = normalize(closest.position - vec3(-1.2, 0.5, -3.0));
            closest.albedo = vec3(0.8, 0.2, 0.2);
            closest.emission = vec3(0.0);
            closest.isEmissive = false;
        }
    }

    // Сфера 2: Светящаяся (эмиссивная) - белая/желтая
    if (intersectSphere(ray, vec3(0.0, 2.5, -3.0), 0.5, t)) {
        if (t < closest.t) {
            closest.hit = true;
            closest.t = t;
            closest.position = ray.origin + ray.direction * t;
            closest.normal = normalize(closest.position - vec3(1.2, 0.5, -3.0));
            closest.albedo = vec3(1.0, 0.9, 0.7);
            closest.emission = vec3(1.0, 0.9, 0.7) * 30.0; // Яркий желтоватый свет
            closest.isEmissive = true;
        }
    }
    
    // Пол
    if (intersectPlane(ray, 0.0, t)) {
        if (t < closest.t) {
            closest.hit = true;
            closest.t = t;
            closest.position = ray.origin + ray.direction * t;
            closest.normal = vec3(0.0, 1.0, 0.0);
            
            // Шахматная текстура
            float checker = mod(floor(closest.position.x) + floor(closest.position.z), 2.0);
            closest.albedo = mix(vec3(0.3), vec3(0.7), checker);
            closest.emission = vec3(0.0);
            closest.isEmissive = false;
        }
    }
    
    return closest;
}

// ===== PATH TRACING =====
vec3 pathTrace(Ray ray, int maxBounces) {
    vec3 color = vec3(0.0);
    vec3 throughput = vec3(1.0);
    
    for (int bounce = 0; bounce < maxBounces; bounce++) {
        HitInfo hit = traceScene(ray);

        if (!hit.hit) {
            // Небо (ambient)
            color += throughput * vec3(0.0, 0.0, 0.0001) * 0.3;
            break;
        }

        // Добавляем эмиссию
        if (hit.isEmissive) {
            color += throughput * hit.emission;
            break; // Светящиеся объекты не отражают свет дальше
        }
        
        // Russian Roulette (для оптимизации)
        float p = max(throughput.r, max(throughput.g, throughput.b));
        if (randomFloat() > p) {
            break;
        }
        throughput /= p;
        
        // Диффузное отражение
        vec3 newDirection = normalize(hit.normal + randomInUnitSphere());

        // Обновляем throughput (BRDF * cos(theta) / pdf)
        float cosTheta = max(0.0, dot(hit.normal, newDirection));
        throughput *= hit.albedo * cosTheta * 2.0; // 2.0 - упрощенная компенсация
        
        // Новый луч
        ray.origin = hit.position + hit.normal * 0.001;
        ray.direction = newDirection;
    }
    
    return color;
}

struct Camera {
    mat4 view;
    mat4 proj;
    vec4 pos;
};

layout(set = 0, binding = 0) uniform CameraBuffer {
    Camera camera;
};

// ===== ГЕНЕРАЦИЯ ЛУЧА ИЗ КАМЕРЫ =====
Ray generateCameraRay(vec2 uv) {
    // Инвертируем view и projection матрицы
    mat4 invView = inverse(camera.view);
    mat4 invProj = inverse(camera.proj);
    
    // UV в NDC space
    vec4 clipSpace = vec4(uv, -1.0, 1.0);

    // NDC -> View space
    vec4 viewSpace = invProj * clipSpace;
    viewSpace = vec4(viewSpace.xy, -1.0, 0.0);
    
    // View space -> World space
    vec4 worldSpace = invView * viewSpace;

    Ray ray;
    // Позиция камеры из inverse view matrix
    ray.origin = (invView * vec4(0.0, 0.0, 0.0, 1.0)).xyz;
    ray.direction = normalize(worldSpace.xyz);
    
    return ray;
}

void main() {

    vec2 resolution = vec2(800.0, 600.0);
    rngState = uint(gl_FragCoord.x) * 1973u + uint(gl_FragCoord.y) * 9277u + 12345u;
    vec3 finalColor = vec3(0.0);
    int samples = 30;

    for (int i = 0; i < samples; i++) {
        vec2 jitter = vec2(randomFloat(), randomFloat()) - 0.5;
        vec2 uv = (gl_FragCoord.xy + jitter) / resolution;
        uv = uv * 2.0 - 1.0;
        uv.x *= resolution.x / resolution.y;

        Ray ray = generateCameraRay(uv);
        finalColor += pathTrace(ray, 5);
    }

    finalColor /= float(samples);

    finalColor = finalColor / (finalColor + vec3(1.0));
    finalColor = pow(finalColor, vec3(1.0 / 2.2));

    outColor = vec4(finalColor, 1.0);
}