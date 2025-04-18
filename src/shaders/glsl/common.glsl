struct Camera {
    vec3 position;
    vec3 uu;
    vec3 vv;
    vec3 ww;
};

struct Ray {
    vec3 origin;
    vec3 direction;
};

struct Hit {
    float dist;
    uint material_index;
    vec3 color;
    bool hit;
};

struct DirectionalLight {
    vec3 direction;
    vec3 color;
    float intensity;
};

struct Material {
    float specular;
    float shininess;
    float roughness;
    vec3 color;
};

float sphere_sdf(vec3 p, float r) {
    return length(p) - r;
}

float box_sdf(vec3 p, vec3 dimension, float corner_radius) {
    vec3 q = abs(p) - dimension + corner_radius;
    return length(max(vec3(0), q)) + min(max(q.x, max(q.y, q.z)), 0.0) - corner_radius;
}

float cylinder_sdf(vec3 p, float radius, float height, float corner_radius) {
    vec2 d = vec2(p.xz.length(), abs(p.y)) - vec2(radius, height * 0.5) + corner_radius;
    return (max(d, vec2(0))).length() + min(max(d.x, d.y), 0.0) - corner_radius;
}

float line_sdf(vec3 p, vec3 a, vec3 b, float r) {
    vec3 pa = p - a;
    vec3 ba = b - a;
    float h = min(1.0, max(0.0, dot(pa, ba) / dot(ba, ba)));
    return (pa - h * ba).length() - r;
}

float smooth_min(float d1, float d2, float k) {
    float h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0., 1.);
    return mix(d2, d1, h) - k * h * (1. - h);
}

vec3 smooth_min_vec4(vec4 d1, vec4 d2, float k) {
    float h = clamp(0.5 + 0.5 * (d2.w - d1.w) / k, 0., 1.);
    return mix(d2.xyz, d1.xyz, h) - k * h * (1. - h);
}

vec3 repeat(vec3 p, float s, float lima, float limb) {
    return vec3(p.x - s * clamp(round(p.x / s), lima, limb), p.y, p.z - s * clamp(round(p.z / s), lima, limb));
}

Material[2] materials;