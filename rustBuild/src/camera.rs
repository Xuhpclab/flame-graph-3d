use cgmath::{prelude::*, vec3, Vector3};
pub struct Camera {
    pub world_up: Vector3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub position: Vector3<f32>,
}
impl Camera {
    pub const LOOK_SPEED: f32 = 0.3;

    pub fn new() -> Self {
        Camera {
            world_up: vec3(0.0, 1.0, 0.0),
            yaw: 1.18,
            pitch: 0.0,
            position: vec3(0.0, 1.0, 0.0),
        }
    }
    pub fn front(&self) -> Vector3<f32> {
        vec3(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }
    pub fn right(&self) -> Vector3<f32> {
        self.front().cross(self.world_up).normalize()
    }
    pub fn up(&self) -> Vector3<f32> {
        self.right().cross(self.front()).normalize()
    }
}
