use raylib::prelude::*;
use std::f32::consts::PI;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

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

use crate::structures::{house_peak, house_roof, house_roof_peak, tree_structure,cave,portal_structure};

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

fn calculate_emissive_lighting(
    intersect: &Intersect,
    objects: &[Cube],
    quality: &QualitySettings,
) -> Vector3 {
    // Solo calcular luz emisiva en calidades bajas para mantener rendimiento
    if quality.shadow_quality < 0.5 || quality.resolution_scale > 0.8 {
        return Vector3::zero(); // Desactivar en ultra y altas calidades
    }

    let mut total_light = Vector3::zero();
    
    for object in objects {
        if object.material.emission_strength > 0.0 {
            // Solo verificar objetos muy cercanos (distancia mucho más corta)
            let object_center = Vector3::new(object.center.x, object.center.y, object.center.z);
            let distance = (object_center - intersect.point).length();
            if distance > 3.0 {  // Solo objetos muy cercanos
                continue;
            }
            
            // Cálculo simple y directo sin múltiples fuentes de luz
            let light_dir = (object_center - intersect.point).normalized();
            
            // Cálculo simple sin shadow casting para mejor rendimiento
            let intensity = (object.material.emission_strength * 0.1) / (distance * distance + 3.0);
            let dot_product = intersect.normal.dot(light_dir).max(0.0);
            let emission_vector = Vector3::new(
                object.material.emission.x,
                object.material.emission.y,
                object.material.emission.z
            );
            let light_contribution = emission_vector * intensity * dot_product;
            total_light = total_light + light_contribution;
        }
    }
    
    total_light
}

fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Cube],
) -> f32 {
    let light_pos = Vector3::new(light.position.x, light.position.y, light.position.z);
    let light_dir = (light_pos - intersect.point).normalized();
    let light_distance = (light_pos - intersect.point).length();
    let shadow_ray_origin = offset_origin(intersect, &light_dir);

    // Usar parallel iterator para verificar intersecciones de sombra
    let has_shadow = objects
        .par_iter()
        .any(|object| {
            let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
            shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance
        });

    if has_shadow { 1.0 } else { 0.0 }
}

pub fn cast_ray(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    objects: &[Cube],
    light: &Light,
    texture_manager: &TextureManager,
    depth: u32,
    quality: &QualitySettings,
) -> Vector3 {
    if depth > quality.max_ray_depth {
        return procedural_sky(*ray_direction);
    }

    // Usar parallel iterator para encontrar la intersección más cercana
    let closest_intersection = objects
        .par_iter()
        .map(|object| object.ray_intersect(ray_origin, ray_direction))
        .filter(|i| i.is_intersecting)
        .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));

    let intersect = match closest_intersection {
        Some(i) => i,
        None => return procedural_sky(*ray_direction),
    };

    let light_pos = Vector3::new(light.position.x, light.position.y, light.position.z);
    let light_dir = (light_pos - intersect.point).normalized();
    let view_dir = (*ray_origin - intersect.point).normalized();
    let reflect_dir = reflect(&-light_dir, &intersect.normal).normalized();

    let shadow_intensity = if quality.shadow_quality > 0.0 {
        cast_shadow(&intersect, light, objects) * quality.shadow_quality
    } else {
        0.0  // Sin sombras para mejorar rendimiento
    };
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    let diffuse_color = if let Some(texture_path) = &intersect.material.texture_id {
        let texture_color = texture_manager.get_pixel_color(texture_path, intersect.u, intersect.v);
        // Brillo extra para lava - autoiluminación simulada
        if texture_path.contains("lava") {
            texture_color * 1.8  // Multiplicar brillo para simular autoiluminación
        } else {
            texture_color
        }
    } else {
        intersect.material.diffuse
    };

    let diffuse_intensity = intersect.normal.dot(light_dir).max(0.0) * light_intensity;
    let mut diffuse = diffuse_color * diffuse_intensity;
    
    // Brillo base para lava (sin depender de la luz)
    if let Some(texture_path) = &intersect.material.texture_id {
        if texture_path.contains("lava") {
            diffuse = diffuse + diffuse_color * 0.6;  // Agregar brillo constante
        }
    }

    // Agregar iluminación de materiales emisivos
    let emissive_light = calculate_emissive_lighting(&intersect, objects, quality);
    diffuse = diffuse + diffuse_color * emissive_light;

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
    let reflect_color = if reflectivity > 0.0 && depth < quality.max_ray_depth {
        let reflect_dir = reflect(ray_direction, &intersect.normal).normalized();
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        cast_ray(&reflect_origin, &reflect_dir, objects, light, texture_manager, depth + 1, quality)
    } else {
        Vector3::zero()
    };

    // Agregar contribución emisiva del material
    let emission = intersect.material.emission * intersect.material.emission_strength;
    
    phong_color * (1.0 - reflectivity) + reflect_color * reflectivity + emission
}

pub fn render(
    framebuffer: &mut Framebuffer,
    objects: &[Cube],
    camera: &Camera,
    light: &Light,
    texture_manager: &TextureManager,
    quality: &QualitySettings,
) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    // Contador para verificar que se usen múltiples threads
    static THREAD_COUNTER: AtomicUsize = AtomicUsize::new(0);
    
    // Crear una lista de todos los píxeles a procesar
    let total_pixels = framebuffer.width * framebuffer.height;
    let pixels: Vec<(u32, Color)> = (0..total_pixels)
        .into_par_iter()
        .map(|pixel_index| {
            // Incrementar contador por thread (solo para debug)
            THREAD_COUNTER.fetch_add(1, Ordering::Relaxed);
            
            let x = pixel_index % framebuffer.width;
            let y = pixel_index / framebuffer.width;
            
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = Vector3::new(screen_x, screen_y, -1.0).normalized();
            let rotated_direction = camera.basis_change(&ray_direction);

            let pixel_color_v3 = cast_ray(&camera.eye, &rotated_direction, objects, light, texture_manager, 0, quality);
            let pixel_color = vector3_to_color(pixel_color_v3);

            (pixel_index, pixel_color)
        })
        .collect();

    // Debug: imprimir cada vez que se renderiza para verificar paralelización
    static RENDER_COUNT: AtomicUsize = AtomicUsize::new(0);
    let count = RENDER_COUNT.fetch_add(1, Ordering::Relaxed);
    if count % 10 == 0 {  // Solo imprimir cada 10 renders para no saturar
        println!("Render #{}: Procesados {} píxeles con {} threads disponibles", 
                 count, 
                 THREAD_COUNTER.load(Ordering::Relaxed),
                 rayon::current_num_threads());
        THREAD_COUNTER.store(0, Ordering::Relaxed); // Reset counter
    }

    // Aplicar los píxeles calculados al framebuffer
    for (pixel_index, color) in pixels {
        let x = pixel_index % framebuffer.width;
        let y = pixel_index / framebuffer.width;
        framebuffer.set_pixel_color(x, y, color);
    }
}

// Configuración de calidad
#[derive(Clone, Copy)]
struct QualitySettings {
    resolution_scale: f32,  // 1.0 = calidad máxima, 0.5 = mitad de resolución
    max_ray_depth: u32,     // Máxima profundidad de recursión de rayos
    shadow_quality: f32,    // Calidad de sombras (0.0 = sin sombras, 1.0 = máxima)
}

impl QualitySettings {
    fn ultra() -> Self { Self { resolution_scale: 1.0, max_ray_depth: 3, shadow_quality: 1.0 } }
    fn high() -> Self { Self { resolution_scale: 0.75, max_ray_depth: 2, shadow_quality: 1.0 } }
    fn medium() -> Self { Self { resolution_scale: 0.5, max_ray_depth: 1, shadow_quality: 0.7 } }
    fn low() -> Self { Self { resolution_scale: 0.33, max_ray_depth: 1, shadow_quality: 0.3 } }
    fn potato() -> Self { Self { resolution_scale: 0.15, max_ray_depth: 0, shadow_quality: 0.0 } }
}

fn main() {
    let window_width = 1300;
    let window_height = 900;
    
    // Inicializar con calidad media para mejor rendimiento
    let mut current_quality = QualitySettings::potato();
 
    let (mut window, thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raytracer 3D - Arrow keys: orbit | W/S: zoom | Q/A: vertical | 1-5: quality")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut texture_manager = TextureManager::new();

    texture_manager.load_texture(&mut window, &thread, "assets/wood.png");
    texture_manager.load_texture(&mut window, &thread, "assets/rock.png");
    texture_manager.load_texture(&mut window, &thread, "assets/log.png");
    texture_manager.load_texture(&mut window, &thread, "assets/log2.png");
    texture_manager.load_texture(&mut window, &thread, "assets/leaf.png");
    texture_manager.load_texture(&mut window, &thread, "assets/grass_low.png");
    texture_manager.load_texture(&mut window, &thread, "assets/soil.png");
    texture_manager.load_texture(&mut window, &thread, "assets/obs.png");
    texture_manager.load_texture(&mut window, &thread, "assets/lava.png");
    
    // Crear framebuffer inicial con calidad media
    let render_width = (window_width as f32 * current_quality.resolution_scale) as u32;
    let render_height = (window_height as f32 * current_quality.resolution_scale) as u32;
    let mut framebuffer = Framebuffer::new(render_width, render_height);

    let brick_material = Material::new(
        Vector3::new(0.8, 0.4, 0.2), // rojizo
        10.0,                        // poco brillo
        [0.9, 0.1, 0.0, 0.0],        // difuso alto, especular bajo, sin reflejo, opaco
        0.0,
        None,
    );

    let wood_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0), // blanco (textura define el color)
        20.0,
        [0.9, 0.1, 0.0, 0.0],        // más difuso, menos especular para madera
        0.0,
        Some("assets/wood.png".to_string()),
    );

    let log_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0), // blanco (textura define el color)
        8.0,                         // menos brillo que madera procesada
        [0.9, 0.1, 0.0, 0.0],       // difuso alto, especular muy bajo para tronco natural
        0.0,
        Some("assets/log.png".to_string()),
    );

    let log2_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0), // blanco (textura define el color)
        8.0,
        [0.9, 0.1, 0.0, 0.0],
        0.0,
        Some("assets/log2.png".to_string()),
    );

    let soil_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0), 
        3.0,                          
        [0.95, 0.05, 0.0, 0.0],      
        0.0,
        Some("assets/soil.png".to_string()),
    );

    let leaf_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0), 
        12.0,                        
        [0.85, 0.15, 0.0, 0.0],      
        0.0,
        Some("assets/leaf.png".to_string()),
    );

    let lava_material = Material::new_emissive(
        Vector3::new(1.5, 1.3, 1.0),  // color base muy claro para simular autoiluminación
        5.0,                          // especular muy bajo para que se vea la textura
        [1.0, 0.0, 0.0, 0.0],         // 100% difuso para mostrar textura sin reflejos
        0.0,
        Some("assets/lava.png".to_string()),
        Vector3::new(0.8, 0.3, 0.05), // Emisión más suave para mejor rendimiento
        2.0,                          // fuerza de emisión reducida
    );

    let grass_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0), 
        10.0,                        
        [0.9, 0.1, 0.0, 0.0],        
        0.0,
        Some("assets/grass_low.png".to_string()),
    );

    let rock_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0), // blanco (textura define el color)
        6.0,                          // muy poco brillo para roca
        [0.95, 0.05, 0.0, 0.0],      // difuso muy alto, especular bajo, sin reflejo, opaco
        0.0,
        Some("assets/rock.png".to_string()),
    );

    let metal_material = Material::new(
        Vector3::new(0.7, 0.7, 0.7), 
        100.0,
        [0.2, 0.6, 0.8, 0.0],       
        0.0,
        None,
    );

    let obs_material = Material::new(
        Vector3::new(0.1, 0.1, 0.15), // negro azulado (como obsidiana real)
        120.0,                        // muy brillante (superficie pulida como cristal)
        [0.3, 0.4, 0.0, 0.0],         // poco difuso, especular alto, muy reflectivo, opaco
        0.0,
        Some("assets/obs.png".to_string()),
    );


    let ice_material = Material::new(
        Vector3::new(0.6, 0.8, 1.0),  // azul bien pronunciado
        50.0,                         // brillo suave
        [0.85, 0.1, 0.25, 0.0],       // MUY difuso, poco especular, reflejo MUY ligero, opaco
        0.0,                          // sin refracción
        None,
    );

    let snow_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0),  // blanco (textura define el color)
        12.0,                         // brillo suave
        [0.85, 0.1, 0.05, 0.0],       // MUY difuso, poco especular, reflejo MUY ligero, opaco
        0.0,                          // sin refracción
        Some("assets/snow.png".to_string()),
    );

    let portal_material = Material::new(
        Vector3::new(0.4, 0.1, 0.8),  // púrpura brillante y vibrante
        80.0,                         // brillante pero no excesivo
        [0.3, 0.2, 0.1, 0.6],         // difuso medio, especular bajo, poco reflejo, TRANSPARENTE
        1.4,                          // índice de refracción (como vidrio especial)
        None,
    );

    let mut objects = Vec::new();


//house base 
    house_structure(&mut objects, rock_material.clone());
    house_roof(&mut objects, log_material.clone());
    house_roof_peak(&mut objects, log_material.clone());
    house_peak(&mut objects, log_material.clone());
    cave(&mut objects, grass_material.clone(), soil_material.clone(), rock_material.clone(), ice_material.clone(), lava_material.clone());
    portal_structure(&mut objects, obs_material.clone(), snow_material.clone(), portal_material.clone());
    tree_structure(&mut objects, leaf_material.clone(), log2_material.clone());
    

    let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, 5.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );
    let rotation_speed = PI / 100.0;
    let zoom_speed = 0.1;

    let light = Light::new(
        Vec3::new(8.0, 22.0, 20.0),  // Más alta para cubrir también la parte superior
        Color::new(255, 255, 255, 255),
        3.0,  // Reducimos intensidad para que se vea más la luz emisiva
    );

    while !window.window_should_close() {
        let mut quality_changed = false;
        
        // Controles de cámara
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
        if window.is_key_down(KeyboardKey::KEY_Q) {
            camera.move_vertical(zoom_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_A) {
            camera.move_vertical(-zoom_speed);
        }
        
        // Controles de calidad (teclas 1-5)
        if window.is_key_pressed(KeyboardKey::KEY_ONE) {
            current_quality = QualitySettings::potato();
            quality_changed = true;
            println!("Calidad: POTATO (máximo rendimiento)");
        }
        if window.is_key_pressed(KeyboardKey::KEY_TWO) {
            current_quality = QualitySettings::low();
            quality_changed = true;
            println!("Calidad: LOW");
        }
        if window.is_key_pressed(KeyboardKey::KEY_THREE) {
            current_quality = QualitySettings::medium();
            quality_changed = true;
            println!("Calidad: MEDIUM");
        }
        if window.is_key_pressed(KeyboardKey::KEY_FOUR) {
            current_quality = QualitySettings::high();
            quality_changed = true;
            println!("Calidad: HIGH");
        }
        if window.is_key_pressed(KeyboardKey::KEY_FIVE) {
            current_quality = QualitySettings::ultra();
            quality_changed = true;
            println!("Calidad: ULTRA (máxima calidad)");
        }
        
        // Recrear framebuffer si cambió la calidad
        if quality_changed {
            let render_width = (window_width as f32 * current_quality.resolution_scale) as u32;
            let render_height = (window_height as f32 * current_quality.resolution_scale) as u32;
            framebuffer = Framebuffer::new(render_width, render_height);
        }

        render(&mut framebuffer, &objects, &camera, &light, &texture_manager, &current_quality);
        
        // Mostrar FPS para medir el rendimiento
        framebuffer.swap_buffers(&mut window, &thread, true, window_width, window_height);
    }
}
