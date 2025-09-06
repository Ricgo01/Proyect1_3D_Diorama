use raylib::prelude::*;
use std::f32::consts::PI;

mod framebuffer;
mod cube;
mod camera;
mod light;
mod color;

use framebuffer::Framebuffer;
use cube::{Cube, Vec3};
use camera::Camera;
use light::Light;
use color::Color;

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    *incident - *normal * (2.0 * incident.dot(*normal))
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Cube],
    light: &Light,
) -> Color {
    let mut hit_distance = f32::INFINITY;
    let mut hit_cube: Option<&Cube> = None;
    let mut hit_normal = Vec3::new(0.0, 0.0, 0.0);
    let mut hit_point = Vec3::new(0.0, 0.0, 0.0);

    for cube in objects {
        if let Some((t, normal)) = cube.ray_intersect(ray_origin, ray_direction) {
            if t < hit_distance && t > 0.0 {
                hit_distance = t;
                hit_cube = Some(cube);
                hit_normal = normal;
                hit_point = *ray_origin + *ray_direction * t;
            }
        }
    }

    if hit_cube.is_none() {
        return Color::new(0.1, 0.2, 0.5);
    }

    let light_dir = (light.position - hit_point).normalize();
    let diffuse_intensity = hit_normal.dot(light_dir).max(0.0) * light.intensity;
    
    let base_color = Color::new(0.6, 0.3, 0.8);
    let lit_color = Color::new(
        base_color.r * light.color.r * (light.ambient_intensity + diffuse_intensity),
        base_color.g * light.color.g * (light.ambient_intensity + diffuse_intensity),
        base_color.b * light.color.b * (light.ambient_intensity + diffuse_intensity),
    );

    lit_color
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Cube], camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = Vec3::new(screen_x, screen_y, -1.0).normalize();
            let rotated_direction = camera.basis_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light);

            framebuffer.set_current_color(pixel_color.to_raylib_color());
            framebuffer.set_pixel(x, y);
        }
    }
}

fn main() {
    let window_width = 1300;
    let window_height = 900;
 
    let (mut window, thread) = raylib::init()
        .size(window_width, window_height)
        .title("Cube Raytracer")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);
    framebuffer.set_background_color(raylib::prelude::Color::new(25, 51, 127, 255));

    let objects = [
        Cube::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
        Cube::new(Vec3::new(2.5, 0.0, -2.0), Vec3::new(0.8, 0.8, 0.8)),
        Cube::new(Vec3::new(-2.0, 1.0, -1.0), Vec3::new(0.6, 0.6, 0.6)),
    ];

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    let rotation_speed = PI / 100.0;

    let light = Light::new(
        Vec3::new(5.0, 5.0, 5.0),
        Color::new(1.0, 1.0, 1.0),
        1.5,
        0.2
    );

    while !window.window_should_close() {
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            camera.orbit(rotation_speed, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_UP) {
            camera.orbit(0.0, -rotation_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            camera.orbit(0.0, rotation_speed);
        }

        framebuffer.clear();
        render(&mut framebuffer, &objects, &camera, &light);
        framebuffer.swap_buffers(&mut window, &thread, false);
    }
}
