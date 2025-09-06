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
        return Color::new(0.02, 0.05, 0.15);
    }

    let light_dir = (light.position - hit_point).normalize();
    let diffuse_intensity = hit_normal.dot(light_dir).max(0.0) * light.intensity;
    
    let base_color = Color::new(0.6, 0.3, 0.8);
    let lit_color = Color::new(
        base_color.r * light.color.r * (light.ambient + diffuse_intensity),
        base_color.g * light.color.g * (light.ambient + diffuse_intensity),
        base_color.b * light.color.b * (light.ambient + diffuse_intensity),
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
        framebuffer.swap_buffers(&mut window, &thread);
    }
}

        framebuffer.clear();

        let center = Vec3::new(0.0, 0.0, 0.0);
        let cam_pos = center + Vec3::new(
            dist * yaw.cos() * pitch.cos(),
            dist * pitch.sin(),
            dist * yaw.sin() * pitch.cos(),
        );

        render_cube_with_lighting(&mut framebuffer, &cube, cam_pos, &sun_light);

        framebuffer.swap_buffers(&mut rl, &thread, true);
    }
}

fn project_point(point: Vec3, cam_pos: Vec3) -> Option<Vector2> {
    let center = Vec3::new(0.0, 0.0, 0.0);
    let forward = (center - cam_pos).normalized();
    let world_up = Vec3::new(0.0, 1.0, 0.0);
    let right = forward.cross(world_up).normalized();
    let up = right.cross(forward).normalized();

    let relative = point - cam_pos;
    
    let depth = relative.dot(forward);
    if depth <= 0.0 { return None; }
    
    let x_cam = relative.dot(right);
    let y_cam = relative.dot(up);
    
    let fov = 60.0_f32.to_radians();
    let half_tan = (fov * 0.5).tan();
    let aspect = WIDTH as f32 / HEIGHT as f32;
    
    let ndc_x = x_cam / (depth * half_tan * aspect);
    let ndc_y = y_cam / (depth * half_tan);
    
    let screen_x = (ndc_x * 0.5 + 0.5) * WIDTH as f32;
    let screen_y = (-ndc_y * 0.5 + 0.5) * HEIGHT as f32;
    
    Some(Vector2::new(screen_x, screen_y))
}

fn draw_triangle_filled(framebuffer: &mut Framebuffer, p1: Vector2, p2: Vector2, p3: Vector2, color: Color) {
    let min_x = (p1.x.min(p2.x.min(p3.x)).floor() as i32).max(0);
    let max_x = (p1.x.max(p2.x.max(p3.x)).ceil() as i32).min(framebuffer.width as i32 - 1);
    let min_y = (p1.y.min(p2.y.min(p3.y)).floor() as i32).max(0);
    let max_y = (p1.y.max(p2.y.max(p3.y)).ceil() as i32).min(framebuffer.height as i32 - 1);

    let area = (p2.x - p1.x) * (p3.y - p1.y) - (p3.x - p1.x) * (p2.y - p1.y);
    if area.abs() < 1e-6 { return; }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let w1 = ((p2.x - px) * (p3.y - py) - (p3.x - px) * (p2.y - py)) / area;
            let w2 = ((p3.x - px) * (p1.y - py) - (p1.x - px) * (p3.y - py)) / area;
            let w3 = 1.0 - w1 - w2;

            if w1 >= 0.0 && w2 >= 0.0 && w3 >= 0.0 {
                framebuffer.set_pixel_color(x as u32, y as u32, color.to_raylib_color());
            }
        }
    }
}

fn render_cube(framebuffer: &mut Framebuffer, camera: Vec3, light: &Light) {
    let center = Vec3::new(0.0, 0.0, 0.0);
    let half = Vec3::new(0.5, 0.5, 0.5);

    let vertices = vec![
        Vec3::new(center.x - half.x, center.y - half.y, center.z - half.z),
        Vec3::new(center.x + half.x, center.y - half.y, center.z - half.z),
        Vec3::new(center.x + half.x, center.y + half.y, center.z - half.z),
        Vec3::new(center.x - half.x, center.y + half.y, center.z - half.z),
        Vec3::new(center.x - half.x, center.y - half.y, center.z + half.z),
        Vec3::new(center.x + half.x, center.y - half.y, center.z + half.z),
        Vec3::new(center.x + half.x, center.y + half.y, center.z + half.z),
        Vec3::new(center.x - half.x, center.y + half.y, center.z + half.z),
    ];
}

fn render_cube_with_lighting(framebuffer: &mut Framebuffer, cube: &Cube, cam_pos: Vec3, light: &Light) {
    let half = cube.half;
    let center = cube.center;
    
    let vertices = [
        Vec3::new(center.x - half.x, center.y - half.y, center.z - half.z),
        Vec3::new(center.x + half.x, center.y - half.y, center.z - half.z),
        Vec3::new(center.x + half.x, center.y + half.y, center.z - half.z),
        Vec3::new(center.x - half.x, center.y + half.y, center.z - half.z),
        Vec3::new(center.x - half.x, center.y - half.y, center.z + half.z),
        Vec3::new(center.x + half.x, center.y - half.y, center.z + half.z),
        Vec3::new(center.x + half.x, center.y + half.y, center.z + half.z),
        Vec3::new(center.x - half.x, center.y + half.y, center.z + half.z),
    ];
    
    let mut projected: Vec<Option<Vector2>> = Vec::new();
    for vertex in vertices.iter() {
        projected.push(project_point(*vertex, cam_pos));
    }

    let faces = [
        ((0, 1, 2), (0, 2, 3), Vec3::new(0.0, 0.0, -1.0), 
         Vec3::new(center.x, center.y, center.z - half.z), Color::light_purple()),
        
        ((4, 7, 6), (4, 6, 5), Vec3::new(0.0, 0.0, 1.0), 
         Vec3::new(center.x, center.y, center.z + half.z), Color::dark_purple()),
         
        ((0, 3, 7), (0, 7, 4), Vec3::new(-1.0, 0.0, 0.0), 
         Vec3::new(center.x - half.x, center.y, center.z), Color::medium_purple()),
         
        ((1, 5, 6), (1, 6, 2), Vec3::new(1.0, 0.0, 0.0), 
         Vec3::new(center.x + half.x, center.y, center.z), Color::medium_purple()),
         
        ((0, 4, 5), (0, 5, 1), Vec3::new(0.0, -1.0, 0.0), 
         Vec3::new(center.x, center.y - half.y, center.z), Color::purple()),
         
        ((3, 2, 6), (3, 6, 7), Vec3::new(0.0, 1.0, 0.0), 
         Vec3::new(center.x, center.y + half.y, center.z), Color::purple()),
    ];

        
    for &((i1, i2, i3), (i4, i5, i6), normal, face_center, base_color) in faces.iter() {
        let light_intensity = light.calculate_lighting(face_center, normal);
        
        let lit_color = Color::new(
            base_color.r * light.color.r * light_intensity,
            base_color.g * light.color.g * light_intensity,
            base_color.b * light.color.b * light_intensity,
        );
        
        if let (Some(p1), Some(p2), Some(p3)) = (projected[i1], projected[i2], projected[i3]) {
            draw_triangle_filled(framebuffer, p1, p2, p3, lit_color);
        }
        
        if let (Some(p1), Some(p2), Some(p3)) = (projected[i4], projected[i5], projected[i6]) {
            draw_triangle_filled(framebuffer, p1, p2, p3, lit_color);
        }
    }
}
