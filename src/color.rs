#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { 
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0), 
            b: b.clamp(0.0, 1.0)
        }
    }

    pub fn from_u8(r: u8, g: u8, b: u8) -> Self {
        Self::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
        )
    }

    pub fn to_raylib_color(&self) -> raylib::prelude::Color {
        raylib::prelude::Color::new(
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            255,
        )
    }

    pub fn with_intensity(&self, intensity: f32) -> Color {
        let intensity = intensity.clamp(0.0, 1.0);
        Color::new(
            self.r * intensity,
            self.g * intensity,
            self.b * intensity,
        )
    }

    pub fn purple() -> Self { Self::from_u8(128, 0, 128) }
    pub fn light_purple() -> Self { Self::from_u8(210, 170, 255) }
    pub fn dark_purple() -> Self { Self::from_u8(70, 40, 110) }
    pub fn medium_purple() -> Self { Self::from_u8(150, 90, 210) }
    pub fn black() -> Self { Self::new(0.0, 0.0, 0.0) }
    pub fn white() -> Self { Self::new(1.0, 1.0, 1.0) }
}
