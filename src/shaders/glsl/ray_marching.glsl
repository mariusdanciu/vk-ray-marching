
#include <common.glsl>

vec3 run(vec2 coord, vec2 screen, Camera camera) {
    vec3 col = vec3(coord.x / screen.x, coord.y / screen.y, 0.0);
    col = pow(col, vec3(0.4545));
    return col;
}