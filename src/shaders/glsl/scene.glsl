#include <common.glsl>

Hit sdf(Ray ray, float t) {

    vec3 p = ray.origin + ray.direction * t;

    float d1 = p.y;
    float d4 = 0;
    float d = d1;
    vec3 id = vec3(0);

    {
        vec3 q = p;
        id = round((q - vec3(-0.5, 1.5, -1.)) / 3);
        float d2 = sphere_sdf(repeat_xz(q - vec3(-0.5, 1.0, -1.), 3, -1, 1), 0.5);
        float d3 = sphere_sdf(repeat_xz(q - vec3(-0.5, 1.5, -1.), 3, -1, 1), 0.1);
        d4 = smooth_min(d3, d2, 0.7);
        float d5 = box_sdf(repeat_xz(q - vec3(-0.5, 0.0, -1.), 3, -1, 1), vec3(1., 0.5, 1.), 0.2);
        d5 = smooth_min(d5, d1, 0.4);
        d = min(d4, d5);
    }


    vec3 col = vec3(0, 0, 0);

    int material = 1;
    if(d == d4) {
        material = 0;
    }

    col = materials[material].color;

    if (d == d4) {
        col = abs(id) + 0.3;
    }

    return Hit(d, material, col, true);
}