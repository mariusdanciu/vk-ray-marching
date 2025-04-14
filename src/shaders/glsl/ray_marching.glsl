
#include <common.glsl>

#define MAX_STEPS 300
#define HIT_PRECISION 0.001
#define MAX_DISTANCE 100.0

Hit sdf(Ray ray, float t) {
    float d = 1000.;

    vec3 p = ray.origin + ray.direction * t;

    float d1 = p.y;
    float d2 = sphere_sdf(p - vec3(-0.5, 0.5, 3.), 0.5);
    d = d1;

    d = min(d, d2);
    vec3 col = vec3(0, 0, 0);

    if(d == d2) {
        col = vec3(0.8, 0, 0);
    } else if(d == d1) {
        col = vec3(0.4, 0.4, 0.4);
    }

    return Hit(d, 0, col, true);
}

vec3 normal(vec3 p) {
    float k = 0.5773 * 0.0005;
    vec2 e = vec2(1., -1.);

    vec3 xyy = vec3(e.x, e.y, e.y);
    vec3 yyx = vec3(e.y, e.y, e.x);
    vec3 yxy = vec3(e.y, e.x, e.y);
    vec3 xxx = vec3(e.x, e.x, e.x);

    Ray r_xyy = Ray(p, xyy);
    Ray r_yyx = Ray(p, yyx);
    Ray r_yxy = Ray(p, yxy);
    Ray r_xxx = Ray(p, xxx);

    return normalize(xyy * sdf(r_xyy, k).dist + yyx * sdf(r_yyx, k).dist + yxy * sdf(r_yxy, k).dist + xxx * sdf(r_xxx, k).dist);
}

float occlusion(vec3 pos, vec3 nor) {
    float occ = 0.0;
    float sca = 1.0;
    for (int i = 0; i < 5; i++) {
        float hr = 0.02 + 0.025 * (i * i);

        
        Hit hit = sdf(Ray(pos, nor), hr);

        occ += -(hit.dist - hr) * sca;
        sca *= 0.85;
    }
    return 1.0 - clamp(occ, 0.0, 1.0);
}

float shadow(Ray ray, float k) {
    float res = 1.0;

    float t = 0.01;

    for (int i = 0; i < 64; i++) {
        vec3 pos = ray.origin + ray.direction * t;
        float h = sdf(ray, t).dist;

        res = min(res, k * (max(h, 0.0) / t));
        if (res < 0.0001) {
            break;
        }
        t += clamp(h, 0.01, 5.0);
    }

    return res;
}

Hit ray_march(Ray ray) {
    float t = 0.0;
    for(int i = 0; i < MAX_STEPS; i++) {
        if(t > MAX_DISTANCE) {
            break;
        }

        Hit h = sdf(ray, t);
        t += h.dist;
        if(h.dist < HIT_PRECISION) {
            return Hit(t, h.material_index, h.color, true);
        }
    }
    return Hit(t, 0, vec3(0), false);
}

vec3 path_trace(Ray ray, DirectionalLight d_light, vec3 res, vec3 sky) {
    Hit hit = ray_march(ray);
    
    if (hit.hit) {
        vec3 p = ray.origin + ray.direction * hit.dist;
        vec3 n = normal(p);
        vec3 light_dir = -d_light.direction;
        float occlusion = occlusion(p, n);

        vec3 col = hit.color;

        float sun = clamp(dot(n, light_dir), 0.0, 1.0);
        float shd = shadow(Ray(p + n * 0.0001, light_dir), 32.);

        vec3 light = sun * d_light.color;
        
        light *= occlusion;
        light *= shd;

        // light += shadow * d_light.color; 

        col *= light;
        return col;
    }

    return res;
}

vec3 run(vec2 coord, vec2 screen, Camera camera) {
    vec2 p = (coord - 0.5 * screen) / screen.y;
    p.y = -p.y;

    Ray ray = Ray(camera.position, normalize(p.x * camera.uu + p.y * camera.vv + 1.5 * camera.ww));
    DirectionalLight d_light = DirectionalLight(normalize(vec3(-3., -1.5, -2.)), vec3(1., 0.85, 0.70));

    vec3 sky = clamp(vec3(0.5, 0.8, 1.) - (0.7 * ray.direction.y), 0.0, 1.0);

    sky = mix(
        sky,
        vec3(0.5, 0.7, 0.9),
        exp(-10.0 * max(ray.direction.y, 0.0))
    );

    vec3 res = sky;

    float sundot = clamp(dot(ray.direction, -d_light.direction), 0.0, 1.0);

    res += 0.25 * vec3(1.0, 0.7, 0.4) * pow(sundot, 5.0);
    res += 0.25 * vec3(1.0, 0.6, 0.6) * pow(sundot, 64.0);
    res += 0.25 * vec3(1.0, 0.9, 0.6) * pow(sundot, 512.0);

    res = path_trace(ray, d_light, res, sky);

    res = pow(res, vec3(0.4545));
    return res;
}