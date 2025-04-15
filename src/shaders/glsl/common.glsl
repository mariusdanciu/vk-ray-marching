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
    vec3 color;
};

float sphere_sdf(vec3 p, float r) {
    return length(p) - r;
}

float box_sdf(vec3 p, vec3 dimension, float corner_radius) {
    vec3 q = abs(p) - dimension + corner_radius;
    return max(vec3(0), q).length() + min(max(q.x, max(q.y, q.z)), 0.0) - corner_radius;
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

Material[2] materials;