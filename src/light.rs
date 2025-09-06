use crate::cube::Vec3;
use crate::color::Color;

pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub ambient_intensity: f32,
}

impl Light {
    pub fn new(position: Vec3, color: Color, intensity: f32, ambient_intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity: intensity.clamp(0.0, 1.0),
            ambient_intensity: ambient_intensity.clamp(0.0, 1.0),
        }
    }

    pub fn calculate_lighting(&self, surface_point: Vec3, surface_normal: Vec3) -> f32 {
        let light_direction = (self.position - surface_point).normalized();
        let distance = (self.position - surface_point).length();
        let attenuation = 1.0 / (1.0 + 0.1 * distance + 0.01 * distance * distance);
        let dot_product = surface_normal.dot(light_direction);
        
        let directional_light = if dot_product > 0.0 { 
            dot_product * self.intensity * attenuation 
        } else { 
            0.0 
        };
        
        let total_light = self.ambient_intensity + directional_light;
        total_light.min(1.0)
    }
}