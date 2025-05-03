#include <common.glsl>

Hit sdf(Ray ray, float t) {

    vec3 p = ray.origin + ray.direction * t;

    float d1 = p.y;
    float d4 = 0;
    float d5 = 0;
    float d = d1;
    vec3 id = vec3(0);

    {
        vec3 q = p;
        id = round((q - vec3(-0.5, 1.5, -1.)) / 3);
        float d2 = sphere_sdf(repeat_xz(q - vec3(-0.5, 1.0, -1.), 3, -1, 1), 0.5);
        float d3 = sphere_sdf(repeat_xz(q - vec3(-0.5, 1.5, -1.), 3, -1, 1), 0.1);
        d4 = smooth_min(d3, d2, 0.7);
        d5 = box_sdf(repeat_xz(q - vec3(-0.5, 0.0, -1.), 3, -1, 1), vec3(1., 0.5, 1.), 0.2);
        d5 = smooth_min(d5, d1, 0.4);
        d = min(d4, d5);
    }

    //vec3 col = vec3(0, 0, 0);
    int material = 1;
    vec3 col = materials[material].color;

    if(d == d4) {
        material = 0;
        col = abs(id) + 0.3;
    } else if (d == d1 || d == d5) {
        float f = 0.2 * (-1. + 2. * smoothstep(-0.2, 0.2, 28.0 * sin(p.x * 4.) + 28.0 * sin(p.y * 4.) + 28.0 * sin(p.z * 4.)));
        col += 0.4 * f;
    }

    return Hit(d, material, col, true);
}