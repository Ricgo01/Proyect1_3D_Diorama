use raylib::prelude::*;
use std::f32::consts::PI;

mod framebuffer;
mod cube;
mod camera;
mod light;
mod textures;
mod material;
mod ray_intersect;
mod structures;

use structures::house::house_structure;
use framebuffer::Framebuffer;
use cube::{Vec3, Cube};
use camera::Camera;
use light::Light;
use textures::TextureManager;
use material::{Material, vector3_to_color};
use ray_intersect::{Intersect, RayIntersect};

use crate::structures::{house_peak, house_roof, house_roof_peak, tree_structure};

const ORIGIN_BIAS: f32 = 1e-4;

fn procedural_sky(_dir: Vector3) -> Vector3 {
    // Fondo completamente blanco
    Vector3::new(1.0, 1.0, 1.0)
}

fn offset_origin(intersect: &Intersect, direction: &Vector3) -> Vector3 {
    let offset = intersect.normal * ORIGIN_BIAS;
    if direction.dot(intersect.normal) < 0.0 {
        intersect.point - offset
    } else {
        intersect.point + offset
    }
}

fn reflect(incident: &Vector3, normal: &Vector3) -> Vector3 {
    *incident - *normal * 2.0 * incident.dot(*normal)
}

fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Cube],
) -> f32 {
    let light_dir = (light.position.to_vector3() - intersect.point).normalized();
    let light_distance = (light.position.to_vector3() - intersect.point).length();

    let shadow_ray_origin = offset_origin(intersect, &light_dir);

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
            return 1.0;
        }
    }

    0.0
}

pub fn cast_ray(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    objects: &[Cube],
    light: &Light,
    texture_manager: &TextureManager,
    depth: u32,
) -> Vector3 {
    if depth > 2 {
        return procedural_sky(*ray_direction);
    }

    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    if !intersect.is_intersecting {
        return procedural_sky(*ray_direction);
    }

    let light_dir = (light.position.to_vector3() - intersect.point).normalized();
    let view_dir = (*ray_origin - intersect.point).normalized();
    let reflect_dir = reflect(&-light_dir, &intersect.normal).normalized();

    let shadow_intensity = cast_shadow(&intersect, light, objects) * 0.7; // Sombras más suaves = menos cálculo
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    let diffuse_color = if let Some(texture_path) = &intersect.material.texture_id {
        texture_manager.get_pixel_color(texture_path, intersect.u, intersect.v)
    } else {
        intersect.material.diffuse
    };

    let diffuse_intensity = intersect.normal.dot(light_dir).max(0.0) * light_intensity;
    let diffuse = diffuse_color * diffuse_intensity;

    let specular_intensity = view_dir.dot(reflect_dir).max(0.0).powf(intersect.material.specular * 0.8) * light_intensity;
    let light_color_v3 = Vector3::new(
        light.color.r as f32 / 255.0, 
        light.color.g as f32 / 255.0, 
        light.color.b as f32 / 255.0
    );
    let specular = light_color_v3 * specular_intensity;

    let albedo = intersect.material.albedo;
    let phong_color = diffuse * albedo[0] + specular * albedo[1];

    let reflectivity = intersect.material.albedo[2];
    let reflect_color = if reflectivity > 0.0 {
        let reflect_dir = reflect(ray_direction, &intersect.normal).normalized();
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        cast_ray(&reflect_origin, &reflect_dir, objects, light, texture_manager, depth + 1)
    } else {
        Vector3::zero()
    };

    phong_color * (1.0 - reflectivity) + reflect_color * reflectivity
}

pub fn render(
    framebuffer: &mut Framebuffer,
    objects: &[Cube],
    camera: &Camera,
    light: &Light,
    texture_manager: &TextureManager,
) {
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

            let ray_direction = Vector3::new(screen_x, screen_y, -1.0).normalized();
            
            let rotated_direction = camera.basis_change(&ray_direction);

            let pixel_color_v3 = cast_ray(&camera.eye, &rotated_direction, objects, light, texture_manager, 0);
            let pixel_color = vector3_to_color(pixel_color_v3);

            framebuffer.set_current_color(pixel_color);
            framebuffer.set_pixel(x, y);
        }
    }
}

fn main() {
    let window_width = 1300;
    let window_height = 900;
 
    let (mut window, thread) = raylib::init()
        .size(window_width, window_height)
        .title("Cube Raytracer with Textures")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut texture_manager = TextureManager::new();

    texture_manager.load_texture(&mut window, &thread, "assets/wood.png");
    texture_manager.load_texture(&mut window, &thread, "assets/rock.png");
    texture_manager.load_texture(&mut window, &thread, "assets/log.png");
    texture_manager.load_texture(&mut window, &thread, "assets/log2.png");
    texture_manager.load_texture(&mut window, &thread, "assets/leaf.png");
    
    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);

    let brick_material = Material::new(
        Vector3::new(0.8, 0.4, 0.2),
        50.0,
        [0.7, 0.3, 0.0, 0.0],
        0.0,
        None, // Descomenta cuando tengas texturas
    );

    let wood_material = Material::new(
        Vector3::new(0.4, 0.2, 0.1),
        30.0,
        [0.8, 0.2, 0.0, 0.0],
        0.0,
        Some("assets/wood.png".to_string()), // Descomenta cuando tengas texturas
    );

    let log_material = Material::new(
        Vector3::new(0.4, 0.2, 0.1),
        30.0,
        [0.8, 0.2, 0.0, 0.0],
        0.0,
        Some("assets/log.png".to_string()), // Descomenta cuando tengas texturas
    );

    let log2_material = Material::new(
        Vector3::new(0.4, 0.2, 0.1),
        30.0,
        [0.8, 0.2, 0.0, 0.0],
        0.0,
        Some("assets/log2.png".to_string()), // Descomenta cuando tengas texturas
    );

    let leaf_material = Material::new(
        Vector3::new(0.4, 0.2, 0.1),
        30.0,
        [0.8, 0.2, 0.0, 0.0],
        0.0,
        Some("assets/leaf.png".to_string()), // Descomenta cuando tengas texturas
    );

    let rock_material = Material::new(
        Vector3::new(0.0, 0.0, 0.0),
        10.0,
        [0.8, 0.2, 0.0, 0.0],
        0.0,
        Some("assets/rock.png".to_string()), // Descomenta cuando tengas texturas
    );

    let metal_material = Material::new(
        Vector3::new(0.5, 0.5, 0.5),
        100.0,
        [0.3, 0.3, 0.4, 0.0],
        0.0,
        None,
    );

    let mut objects = Vec::new();


//house base 
    house_structure(&mut objects, rock_material.clone());
    house_roof(&mut objects, log_material.clone());
    house_roof_peak(&mut objects, log_material.clone());
    house_peak(&mut objects, log_material.clone());

//tree
    tree_structure(&mut objects, leaf_material.clone(), log2_material.clone());
    

    let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, 5.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );
    let rotation_speed = PI / 100.0;
    let zoom_speed = 0.1;

    let light = Light::new(
        Vec3::new(5.0, 5.0, 5.0),
        Color::new(255, 255, 255, 255),
        1.5,
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
        if window.is_key_down(KeyboardKey::KEY_W) {
            camera.zoom(zoom_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_S) {
            camera.zoom(-zoom_speed);
        }

        if camera.is_changed() {
            render(&mut framebuffer, &objects, &camera, &light, &texture_manager);
        }
        
        framebuffer.swap_buffers(&mut window, &thread, false);
    }
}
