use glam::{ivec2, uvec2, vec2, vec3, Mat4, UVec2, Vec2, Vec3, Vec4, Vec4Swizzles};

static DEGREES: f32 = std::f32::consts::PI / 180.;
static UP: Vec3 = vec3(0., 1., 0.);

#[derive(Debug, Clone)]
pub struct Camera {
    pub resolution: Vec2,
    pub position: Vec3,
    pub uu: Vec3,
    pub vv: Vec3,
    pub ww: Vec3,
}

pub enum CameraEvent {
    Resize { w: usize, h: usize },
    RotateXY { delta: Vec2 },
    Up,
    Down,
    Left,
    Right,
}

impl Camera {
    pub fn new_with_pos(position: Vec3, forward: Vec3) -> Camera {
        let ww = forward.normalize();
        let uu = ww.cross(vec3(0., 1., 0.)).normalize();
        let vv = uu.cross(ww).normalize();

        Camera {
            resolution: vec2(800., 600.),
            position,
            uu,
            vv,
            ww,
        }
    }

    pub fn update(&mut self, events: &Vec<CameraEvent>, ts: f32) {
        let speed = 2.;
        let rotation_speed = 2.;
        for event in events {
            match event {
                CameraEvent::Up => self.position += self.ww * speed * ts,
                CameraEvent::Down => self.position -= self.ww * speed * ts,
                CameraEvent::Left => self.position -= self.uu * speed * ts,
                CameraEvent::Right => self.position += self.uu * speed * ts,
                CameraEvent::Resize { w, h } => {
                    self.resolution = vec2(*w as f32, *h as f32);
                }

                CameraEvent::RotateXY { delta } => {
                    let pitch_delta = -delta.y * rotation_speed;
                    let yaw_delta = -delta.x * rotation_speed;

                    let rotation = Mat4::from_rotation_x(pitch_delta as f32 * DEGREES)
                        * Mat4::from_rotation_y(yaw_delta as f32 * DEGREES);

                    let fd = rotation * Vec4::new(self.ww.x, self.ww.y, self.ww.z, 1.);

                    self.ww = fd.xyz().normalize();
                    self.uu = self.ww.cross(vec3(0., 1., 0.)).normalize();
                    self.vv = self.uu.cross(self.ww).normalize();
                }
            }
        }
    }
}
