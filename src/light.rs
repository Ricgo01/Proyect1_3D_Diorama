use raylib::prelude::Color;
use crate::cube::Vec3;

pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
}

impl Light {
    pub fn new(position: Vec3, color: Color, intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity: intensity.clamp(0.0, 2.0),
        }
    }
}