
#include <common.glsl>

#define MAX_STEPS 300
#define HIT_PRECISION 0.001
#define MAX_DISTANCE 100.0

vec3 repeat(vec3 pos, float offset) {
    return vec3(mod(pos.x + offset * 0.5, offset) - offset * 0.5, pos.y, mod(pos.z + offset * 0.5, offset) - offset * 0.5);
}

vec3 opRepLim(vec3 p, float s, float lima, float limb) {
    return vec3(p.x - s * clamp(round(p.x / s), lima, limb), p.y, p.z - s * clamp(round(p.z / s), lima, limb));
}

Hit sdf(Ray ray, float t) {

    vec3 p = ray.origin + ray.direction * t;

    float d1 = p.y;
    float d4 = 0;
    float d = d1;

    {
        vec3 q = p;

        float d2 = sphere_sdf(opRepLim(q - vec3(-0.5, 1.0, 3.), 3, -1, 1), 0.5);
        float d3 = sphere_sdf(opRepLim(q - vec3(-0.5, 1.5, 3.), 3, -1, 1), 0.1);
        d4 = smooth_min(d3, d2, 0.7);
        float d5 = box_sdf(opRepLim(q - vec3(-0.5, 0.0, 3.), 3, -1, 1), vec3(1., 0.5, 1.), 0.2);
        d5 = smooth_min(d5, d1, 0.4);
        d = min(d4, d5);
        //d4 += 0.4;d
        //d = min(d, d4);
    }


    vec3 col = vec3(0, 0, 0);

    int material = 1;
    if(d == d4) {
        material = 0;
    } else if(d == d1) {
    }

    col = materials[material].color;

    return Hit(d, material, col, true);
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
    for(int i = 0; i < 5; i++) {
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

    for(int i = 0; i < 64; i++) {
        vec3 pos = ray.origin + ray.direction * t;
        float h = sdf(ray, t).dist;

        res = min(res, k * (max(h, 0.0) / t));
        if(res < 0.0001) {
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

vec3 path_trace(Ray ray, DirectionalLight d_light, vec3 res, vec3 sky, int bounce) {

    vec3 refl_col = vec3(0);
    float refl_roughness = -1.0;
    bool need_mix = false;

    for(int bounce = 0; bounce < 3; bounce++) {

        Hit hit = ray_march(ray);

        if(hit.hit) {
            vec3 p = ray.origin + ray.direction * hit.dist;
            vec3 n = normal(p);
            vec3 light_dir = -d_light.direction;
            float occlusion = occlusion(p, n);
            float shadow = shadow(Ray(p + n * 0.0001, light_dir), 32);

            vec3 half_angle = normalize(-ray.direction + light_dir);

            Material material = materials[hit.material_index];
            float mat_specular = material.specular;
            float mat_shininess = material.shininess;

            vec3 col = hit.color;

            float shininess = pow(max(dot(n, half_angle), 0.), mat_shininess);

            float sun = clamp(dot(n, light_dir), 0.0, 1.0);
            float indirect = 0.1 * clamp(dot(n, normalize(light_dir * vec3(-1.0, 0.0, -1.0))), 0.0, 1.0);

            vec3 light = sun * d_light.color * pow(vec3(shadow), vec3(1.3, 1.2, 1.5));

            light += sky * vec3(0.16, 0.20, 0.28) * occlusion;
            light += indirect * vec3(0.40, 0.28, 0.20) * occlusion;
            light += mat_specular * shininess * shadow;

            col *= light * d_light.intensity;

            res = clamp(col, 0.0, 1.0);

            if (refl_roughness >= 0) {
                res = mix(res, refl_col, refl_roughness);
            }

            if (material.roughness < 1.0) {
                vec3 refl = normalize(reflect(ray.direction, n));
                ray = Ray(p + n * 0.01, refl);
                refl_col = res;
                refl_roughness = material.roughness;
            } else {
                refl_roughness = -1;
            }
        } else {
            if (refl_roughness >= 0) {
                res = mix(res, refl_col, refl_roughness);
            }
            break;
        }
    }
    return res;
}

vec3 run(vec2 coord, vec2 screen, Camera camera) {
    vec2 p = (coord - 0.5 * screen) / screen.y;
    p.y = -p.y;

    Ray ray = Ray(camera.position, normalize(p.x * camera.uu + p.y * camera.vv + 1.5 * camera.ww));
    DirectionalLight d_light = DirectionalLight(normalize(vec3(-3., -1.5, -2.)), vec3(1., 0.85, 0.70), 1.0);

    vec3 sky = clamp(vec3(0.5, 0.8, 1.) - (0.7 * ray.direction.y), 0.0, 1.0);

    sky = mix(sky, vec3(0.5, 0.7, 0.9), exp(-10.0 * max(ray.direction.y, 0.0)));

    vec3 res = sky;

    float sundot = clamp(dot(ray.direction, -d_light.direction), 0.0, 1.0);

    res += 0.25 * vec3(1.0, 0.7, 0.4) * pow(sundot, 5.0);
    res += 0.25 * vec3(1.0, 0.6, 0.6) * pow(sundot, 64.0);
    res += 0.25 * vec3(1.0, 0.9, 0.6) * pow(sundot, 512.0);

    res = path_trace(ray, d_light, res, sky, 0);

    res = pow(res, vec3(0.4545));
    return res;
}