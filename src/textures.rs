use raylib::prelude::*;
use std::collections::HashMap;
use crate::cube::Vec3;

pub struct CpuTexture {
    width: i32,
    height: i32,
    pixels: Vec<Vec3>, // Normalized RGB values
}

impl CpuTexture {
    pub fn from_image(image: &Image) -> Self {
        // Safe: Raylib handles pixel format internally
        let colors = image.get_image_data(); // Vec<Color>
        let pixels = colors
            .iter()
            .map(|c| {
                Vec3::new(
                    c.r as f32 / 255.0,
                    c.g as f32 / 255.0,
                    c.b as f32 / 255.0,
                )
            })
            .collect();

        CpuTexture {
            width: image.width,
            height: image.height,
            pixels,
        }
    }
}

pub struct TextureManager {
    cpu_textures: HashMap<String, CpuTexture>,
    textures: HashMap<String, Texture2D>, // Store GPU textures for rendering
}

impl TextureManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_texture(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        path: &str,
    ) {
        if self.textures.contains_key(path) {
            return;
        }

        let image = Image::load_image(path)
            .unwrap_or_else(|_| panic!("Failed to load image {}", path));

        let texture = rl
            .load_texture_from_image(thread, &image)
            .unwrap_or_else(|_| panic!("Failed to load texture {}", path));

        let cpu_texture = CpuTexture::from_image(&image);

        self.cpu_textures.insert(path.to_string(), cpu_texture);
        self.textures.insert(path.to_string(), texture);
    }

    pub fn get_pixel_color(
        &self,
        path: &str,
        u: f32,
        v: f32,
    ) -> Vector3 {
        if let Some(cpu_texture) = self.cpu_textures.get(path) {
            let x = ((u * cpu_texture.width as f32) as i32).clamp(0, cpu_texture.width - 1);
            let y = ((v * cpu_texture.height as f32) as i32).clamp(0, cpu_texture.height - 1);

            let index = (y * cpu_texture.width + x) as usize;
            if index < cpu_texture.pixels.len() {
                cpu_texture.pixels[index].to_vector3()
            } else {
                Vector3::new(1.0, 1.0, 1.0) // default white
            }
        } else {
            Vector3::new(1.0, 1.0, 1.0) // default white
        }
    }

    pub fn get_texture(
        &self,
        path: &str,
    ) -> Option<&Texture2D> {
        self.textures.get(path)
    }
}

impl Default for TextureManager {
    fn default() -> Self {
        TextureManager {
            cpu_textures: HashMap::new(),
            textures: HashMap::new(),
        }
    }
}
