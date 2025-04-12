
#include <common.glsl>

#define MAX_STEPS 300
#define HIT_PRECISION 0.001
#define MAX_DISTANCE 100.0


Hit sdf(Ray ray, float t) {
    float d = 1000.;
    
    vec3 p = ray.origin + ray.direction * t;

    float d1 = p.y;
    float d2 = sphere_sdf(p - vec3(-0.5, 1.4, 3.), 0.5);
    d = d1;

    d = min(d, d2);
    vec3 col = vec3(0, 0, 0);

    if (d == d2) {
        col = vec3(0.8, 0, 0);
    } else if (d == d1) {
        col = vec3(0.4, 0.4, 0.4);
    }

    return Hit(d, 0, col, true);
}

Hit ray_march(Ray ray) {
    float t = 0.0;
    for (int i = 0; i < MAX_STEPS; i++) {
        if (t > MAX_DISTANCE) {
            break;
        }

        Hit h = sdf(ray, t);
        t += h.dist;
        if (h.dist < HIT_PRECISION) {
            return Hit(t, h.material_index, h.color, true);
        }
    }
    return Hit(t, 0, vec3(0), false);
}


vec3 run(vec2 coord, vec2 screen, Camera camera) {


    vec2 p = (coord - 0.5*screen) / screen.y;
    p.y = - p.y;
    //vec2 p = (-screen + 2.0*(vec2(coord.x, coord.y)))/screen.y;

    Ray ray = Ray(camera.position, normalize(p.x * camera.uu + p.y * camera.vv + 1.5 * camera.ww));

    Hit hit = ray_march(ray);

    vec3 col = vec3(0);
    if (hit.hit) {
        vec3 p = ray.origin + ray.direction * hit.dist;
        col = hit.color;
    }

    
    col = pow(col, vec3(0.4545));
    return col;
}