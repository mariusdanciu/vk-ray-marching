#version 450

#include <ray_marching.glsl>

layout(location = 0) out vec4 f_color;

layout(push_constant) uniform AppData {
    vec2 screen;
    vec3 cam_position;
    vec3 cam_uu;
    vec3 cam_vv;
    vec3 cam_ww;
    Material[2] materials;
} app;


void main() {
    Camera camera = Camera(app.cam_position, app.cam_uu, app.cam_vv, app.cam_ww);
    vec2 coord = gl_FragCoord.xy;
    materials = app.materials;
    vec3 col = run(coord, app.screen, camera);
    f_color = vec4(col, 1.0);
}