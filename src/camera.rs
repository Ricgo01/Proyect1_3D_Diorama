use crate::cube::Vec3;

pub struct Camera {
    pub eye: Vec3,
    pub center: Vec3,
    pub up: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
}

impl Camera {
    pub fn new(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        let mut camera = Camera {
            eye,
            center,
            up,
            forward: Vec3::new(0.0, 0.0, 0.0),
            right: Vec3::new(0.0, 0.0, 0.0),
        };
        camera.update_basis_vectors();
        camera
    }

    pub fn update_basis_vectors(&mut self) {
        self.forward = (self.center - self.eye).normalized();
        
        self.right = self.forward.cross(self.up).normalized();
        
        self.up = self.right.cross(self.forward);
    }

    pub fn orbit(&mut self, yaw: f32, pitch: f32) {
        let relative_pos = self.eye - self.center;
        
        let radius = relative_pos.length();
        
        let current_yaw = relative_pos.z.atan2(relative_pos.x);
        let current_pitch = (relative_pos.y / radius).asin();
        
        let new_yaw = current_yaw + yaw;
        let new_pitch = (current_pitch + pitch).clamp(-1.5, 1.5);
        
        let cos_pitch = new_pitch.cos();
        let new_relative_pos = Vec3::new(
            radius * cos_pitch * new_yaw.cos(),
            radius * new_pitch.sin(),
            radius * cos_pitch * new_yaw.sin(),
        );
        
        self.eye = self.center + new_relative_pos;
        
        self.update_basis_vectors();
    }

    pub fn basis_change(&self, v: &Vec3) -> Vec3 {
        Vec3::new(
            v.x * self.right.x + v.y * self.up.x - v.z * self.forward.x,
            v.x * self.right.y + v.y * self.up.y - v.z * self.forward.y,
            v.x * self.right.z + v.y * self.up.z - v.z * self.forward.z,
        )
    }
}
