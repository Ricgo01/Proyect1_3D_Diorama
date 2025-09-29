// Importaciones principales para el trazador y utilidades concurrentes
use raylib::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};

mod framebuffer;
mod cube;
mod camera;
mod light;
mod textures;
mod material;
mod ray_intersect;
mod structures;

// Estructuras y utilidades propias del proyecto
use structures::house::house_structure;
use framebuffer::Framebuffer;
use cube::{Vec3, Cube};
use camera::Camera;
use light::Light;
use textures::TextureManager;
use material::{Material, vector3_to_color};
use ray_intersect::{Intersect, RayIntersect};

use crate::structures::{house_peak, house_roof, house_roof_peak, tree_structure,cave,portal_structure, farm};

// Constantes globales que controlan ajustes del trazado
const ORIGIN_BIAS: f32 = 1e-4;
const SKY_TEXTURE_PATH: &str = "assets/sky.png";

// Fuente de iluminación secundaria utilizada para bloques emisivos (lava, portal, etc.)
#[derive(Clone)]
struct EmissiveSource {
    position: Vector3,
    color: Vector3,
    strength: f32,
    radius: f32,
}

// Mapea un rayo a la textura del cielo para obtener el color de fondo
fn procedural_sky(dir: Vector3, texture_manager: &TextureManager) -> Vector3 {
    let dir = dir.normalized();
    let theta = dir.y.clamp(-1.0, 1.0).acos();
    let phi = dir.z.atan2(dir.x);
    let u = 1.0 - (phi + PI) / (2.0 * PI);
    let v = theta / PI;

    texture_manager.get_pixel_color(SKY_TEXTURE_PATH, u, v)
}

// Pequeño desplazamiento para evitar cosas raras de las sombras en intersecciones
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

fn refract(incident: &Vector3, normal: &Vector3, eta_t: f32) -> Option<Vector3> {
    if eta_t <= 0.0 {
        return None;
    }

    let mut n = *normal;
    let mut etai = 1.0f32;
    let mut etat = eta_t;
    let mut cosi = incident.dot(n).clamp(-1.0, 1.0);

    if cosi > 0.0 {
        std::mem::swap(&mut etai, &mut etat);
        n = -n;
    } else {
        cosi = -cosi;
    }

    let eta = etai / etat;
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
    if k < 0.0 {
        None
    } else {
        Some((*incident * eta + n * (eta * cosi - k.sqrt())).normalized())
    }
}

fn calculate_emissive_lighting(
    intersect: &Intersect,
    emissive_sources: &[EmissiveSource],
    quality: &QualitySettings,
) -> Vector3 {
    if emissive_sources.is_empty() {
        return Vector3::zero();
    }

    let max_sources = if quality.shadow_quality < 0.35 {
        emissive_sources.len().min(2)
    } else if quality.resolution_scale < 0.4 {
        emissive_sources.len().min(3)
    } else {
        emissive_sources.len()
    };

    let mut total_light = Vector3::zero();

    for source in emissive_sources.iter().take(max_sources) {
        let to_light = source.position - intersect.point;
        let distance = to_light.length();

        if distance > source.radius {
            continue;
        }

        let light_dir = to_light.normalized();
        let dot_product = intersect.normal.dot(light_dir).max(0.0);

        if dot_product <= 0.0 {
            continue;
        }

        let attenuation = 1.0 / (1.0 + 0.35 * distance * distance);
        let falloff = ((source.radius - distance) / source.radius)
            .max(0.0)
            .powf(1.1);
        let intensity = source.strength * attenuation * falloff;

        total_light = total_light + source.color * intensity * dot_product;
    }

    total_light
}

// Verifica si un rayo hacia la luz queda bloqueado por algún cubo
fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Cube],
) -> f32 {
    let light_pos = Vector3::new(light.position.x, light.position.y, light.position.z);
    let light_dir = (light_pos - intersect.point).normalized();
    let light_distance = (light_pos - intersect.point).length();
    let shadow_ray_origin = offset_origin(intersect, &light_dir);


    let has_shadow = objects
        .par_iter()
        .any(|object| {
            let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
            shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance
        });

    if has_shadow { 1.0 } else { 0.0 }
}

// Núcleo del trazador: dispara un rayo y devuelve el color resultante
pub fn cast_ray(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    objects: &[Cube],
    emissive_sources: &[EmissiveSource],
    light: &Light,
    texture_manager: &TextureManager,
    depth: u32,
    quality: &QualitySettings,
) -> Vector3 {
    if depth > quality.max_ray_depth {
        return procedural_sky(*ray_direction, texture_manager);
    }


    let closest_intersection = objects
        .par_iter()
        .map(|object| object.ray_intersect(ray_origin, ray_direction))
        .filter(|i| i.is_intersecting)
        .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));

    let intersect = match closest_intersection {
        Some(i) => i,
        None => return procedural_sky(*ray_direction, texture_manager),
    };

    let light_pos = Vector3::new(light.position.x, light.position.y, light.position.z);
    let light_dir = (light_pos - intersect.point).normalized();
    let view_dir = (*ray_origin - intersect.point).normalized();
    let reflect_dir = reflect(&-light_dir, &intersect.normal).normalized();

    let shadow_intensity = if quality.shadow_quality > 0.0 {
        cast_shadow(&intersect, light, objects) * quality.shadow_quality
    } else {
        0.0  
    };
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    // Determina el color base del material, usando textura o difuso sólido
    let diffuse_color = if let Some(texture_path) = &intersect.material.texture_id {
        let texture_color = texture_manager.get_pixel_color(texture_path, intersect.u, intersect.v);
        if texture_path.contains("lava") {
            texture_color * 1.8  
        } else {
            texture_color
        }
    } else {
        intersect.material.diffuse
    };

    let diffuse_intensity = intersect.normal.dot(light_dir).max(0.0) * light_intensity;
    let mut diffuse = diffuse_color * diffuse_intensity;
    

    if let Some(texture_path) = &intersect.material.texture_id {
        if texture_path.contains("lava") {
            diffuse = diffuse + diffuse_color * 0.6; 
        }
    }


    // Añade la contribución de emisores locales (lava, portal, etc.)
    let emissive_light = calculate_emissive_lighting(&intersect, emissive_sources, quality);
    diffuse = diffuse + diffuse_color * emissive_light;

    // Componente especular del modelo de iluminación
    let specular_intensity = view_dir.dot(reflect_dir).max(0.0).powf(intersect.material.specular * 0.8) * light_intensity;
    let light_color_v3 = Vector3::new(
        light.color.r as f32 / 255.0, 
        light.color.g as f32 / 255.0, 
        light.color.b as f32 / 255.0
    );
    let specular = light_color_v3 * specular_intensity;

    let albedo = intersect.material.albedo;
    let phong_color = diffuse * albedo[0] + specular * albedo[1];

    let reflectivity = intersect.material.albedo[2].clamp(0.0, 1.0);
    // Calcula reflejos recursivos si el material lo requiere
    let reflect_color = if reflectivity > 0.0 && depth < quality.max_ray_depth {
        let reflect_dir = reflect(ray_direction, &intersect.normal).normalized();
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        cast_ray(
            &reflect_origin,
            &reflect_dir,
            objects,
            emissive_sources,
            light,
            texture_manager,
            depth + 1,
            quality,
        )
    } else {
        Vector3::zero()
    };

    let transparency = if quality.enable_refraction {
        intersect.material.albedo[3].clamp(0.0, 1.0)
    } else {
        0.0
    };

    let mut refract_color = Vector3::zero();
    if transparency > 0.0 && depth < quality.max_ray_depth {
        let refr_index = intersect.material.refractive_index.max(1.0);
        let incident_dir = ray_direction.normalized();
        if let Some(refract_dir) = refract(&incident_dir, &intersect.normal, refr_index) {
            let refract_origin = offset_origin(&intersect, &refract_dir);
            refract_color = cast_ray(
                &refract_origin,
                &refract_dir,
                objects,
                emissive_sources,
                light,
                texture_manager,
                depth + 1,
                quality,
            );
        }
    }

    let base_weight = (1.0 - reflectivity - transparency).max(0.0);


    let emission = intersect.material.emission * intersect.material.emission_strength;

    // Mezcla final de difuso, reflejo, refracción y autoemisión
    phong_color * base_weight + reflect_color * reflectivity + refract_color * transparency + emission
}

// Genera la imagen final iterando por cada píxel de la pantalla virtual
pub fn render(
    framebuffer: &mut Framebuffer,
    objects: &[Cube],
    emissive_sources: &[EmissiveSource],
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

    static THREAD_COUNTER: AtomicUsize = AtomicUsize::new(0);
    
    let total_pixels = framebuffer.width * framebuffer.height;
    let pixels: Vec<(u32, Color)> = (0..total_pixels)
        .into_par_iter()
        .map(|pixel_index| {
            THREAD_COUNTER.fetch_add(1, Ordering::Relaxed);
            
            let x = pixel_index % framebuffer.width;
            let y = pixel_index / framebuffer.width;
            
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = Vector3::new(screen_x, screen_y, -1.0).normalized();
            let rotated_direction = camera.basis_change(&ray_direction);

            let pixel_color_v3 = cast_ray(
                &camera.eye,
                &rotated_direction,
                objects,
                emissive_sources,
                light,
                texture_manager,
                0,
                quality,
            );
            let pixel_color = vector3_to_color(pixel_color_v3);

            (pixel_index, pixel_color)
        })
        .collect();


    for (pixel_index, color) in pixels {
        let x = pixel_index % framebuffer.width;
        let y = pixel_index / framebuffer.width;
        framebuffer.set_pixel_color(x, y, color);
    }
}

// Configuración de calidad para el motor, permite escalado y límites de profundidad
#[derive(Clone, Copy)]
struct QualitySettings {
    resolution_scale: f32,  
    max_ray_depth: u32,     
    shadow_quality: f32,    
    enable_refraction: bool,
}

impl QualitySettings {
    fn ultra() -> Self { Self { resolution_scale: 1.0, max_ray_depth: 4, shadow_quality: 1.0, enable_refraction: true } }
    fn high() -> Self { Self { resolution_scale: 0.75, max_ray_depth: 2, shadow_quality: 1.0, enable_refraction: false } }
    fn medium() -> Self { Self { resolution_scale: 0.5, max_ray_depth: 1, shadow_quality: 0.7, enable_refraction: false } }
    fn low() -> Self { Self { resolution_scale: 0.33, max_ray_depth: 1, shadow_quality: 0.3, enable_refraction: false } }
    fn potato() -> Self { Self { resolution_scale: 0.15, max_ray_depth: 0, shadow_quality: 0.0, enable_refraction: false } }
}

fn quality_label_for_key(key: &str) -> &'static str {
    match key {
        "potato" => "POTATO (máximo rendimiento)",
        "low" => "LOW",
        "medium" => "MEDIUM",
        "high" => "HIGH",
        "ultra" => "ULTRA (máxima calidad)",
        _ => "DESCONOCIDA",
    }
}

// Punto de entrada: prepara Raylib, carga recursos y ejecuta el bucle principal
fn main() {
    let window_width = 1300;
    let window_height = 900;
    
    let (mut window, thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raytracer 3D - Arrow keys: orbit | W/S: zoom | Q/A: vertical | 1-5: quality")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut texture_manager = TextureManager::new();

    // Importación de los assets de texturas que se usarán en los materiales
    texture_manager.load_texture(&mut window, &thread, "assets/wood.png");
    texture_manager.load_texture(&mut window, &thread, "assets/rock.png");
    texture_manager.load_texture(&mut window, &thread, "assets/log.png");
    texture_manager.load_texture(&mut window, &thread, "assets/log2.png");
    texture_manager.load_texture(&mut window, &thread, "assets/leaf.png");
    texture_manager.load_texture(&mut window, &thread, "assets/grass.png");
    texture_manager.load_texture(&mut window, &thread, "assets/soil.png");
    texture_manager.load_texture(&mut window, &thread, "assets/obs.png");
    texture_manager.load_texture(&mut window, &thread, "assets/lava.png");
    texture_manager.load_texture(&mut window, &thread, "assets/bamboo.png");
    texture_manager.load_texture(&mut window, &thread, "assets/diamond.png");
    texture_manager.load_texture(&mut window, &thread, "assets/bush.png");
    texture_manager.load_texture(&mut window, &thread, "assets/face.png");
    texture_manager.load_texture(&mut window, &thread, SKY_TEXTURE_PATH);
    
    // Pre-crear framebuffers para cada calidad para cambios instantáneos
    let mut framebuffers: HashMap<&'static str, Framebuffer> = HashMap::new();
    let mut quality_lookup: HashMap<&'static str, QualitySettings> = HashMap::new();
    
    // Crear framebuffers para todas las calidades
    let qualities = [
        ("potato", QualitySettings::potato()),
        ("low", QualitySettings::low()),
        ("medium", QualitySettings::medium()),
        ("high", QualitySettings::high()),
        ("ultra", QualitySettings::ultra()),
    ];
    
    for (name, quality) in &qualities {
        let render_width = (window_width as f32 * quality.resolution_scale) as u32;
        let render_height = (window_height as f32 * quality.resolution_scale) as u32;
        framebuffers.insert(*name, Framebuffer::new(render_width, render_height));
        quality_lookup.insert(*name, *quality);
        println!("Pre-creado framebuffer {}: {}x{}", name, render_width, render_height);
    }

    let default_quality_key = "potato";
    let mut current_framebuffer_key = default_quality_key;
    let mut current_quality = *quality_lookup
        .get(current_framebuffer_key)
        .expect("Calidad inicial no encontrada");


    // Definición de materiales principales usados en las estructuras
    let log_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0), 
        8.0,                         
        [0.9, 0.1, 0.0, 0.0],       
        0.0,
        Some("assets/log.png".to_string()),
    );
    
    let face_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0),
        8.0,
        [1.35, 0.12, 0.0, 0.0],
        0.0,
        Some("assets/face.png".to_string()),
    );

    let bamboo_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0),
        8.0,                         
        [0.9, 0.1, 0.0, 0.0], 
        0.0,      
        Some("assets/bamboo.png".to_string()),
    );

    let log2_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0), 
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

    let bush_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0), 
        12.0,                        
        [0.85, 0.15, 0.0, 0.0],      
        0.0,
        Some("assets/bush.png".to_string()),
    );

    let lava_material = Material::new_emissive(
        Vector3::new(1.5, 1.3, 1.0),  
        5.0,                          
        [1.0, 0.0, 0.0, 0.0],         
        0.0,
        Some("assets/lava.png".to_string()),
        Vector3::new(0.8, 0.3, 0.05), 
        2.0,                          
    );

    let grass_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0), 
        10.0,                        
        [0.9, 0.1, 0.0, 0.0],        
        0.0,
        Some("assets/grass.png".to_string()),
    );

    let diamond_material = Material::new(
        Vector3::new(0.65, 0.92, 1.0),  
        10.0,                          
        [1.1, 0.55, 0.0, 0.0],        
        0.0,                           
        Some("assets/diamond.png".to_string()),
    );

    let rock_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0), 
        6.0,                          
        [0.95, 0.05, 0.0, 0.0],      
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
        Vector3::new(0.1, 0.1, 0.15), 
        120.0,                        
        [0.3, 0.4, 0.0, 0.0],         
        0.0,
        Some("assets/obs.png".to_string()),
    );


    let ice_material = Material::new(
        Vector3::new(0.6, 0.8, 1.0),  
        50.0,                         
        [0.85, 0.1, 0.25, 0.0],     
        0.0,                          
        None,
    );

    let snow_material = Material::new(
        Vector3::new(1.0, 1.0, 1.0),  
        12.0,                         
        [0.85, 0.1, 0.05, 0.0],       
        0.0,                          
        Some("assets/snow.png".to_string()),
    );

    let portal_material = Material::new(
        Vector3::new(0.4, 0.1, 0.8), 
        80.0,                        
        [0.25, 0.2, 0.15, 0.55],     
        1.45,
        None,                         
    );

    let water_material = Material::new(
        Vector3::new(0.18, 0.34, 0.48),  
        70.0,                            
        [0.35, 0.18, 0.12, 0.55],        
        1.33,                            
        None,
    );

    // Colección de cubos que componen el mundo voxel
    let mut objects = Vec::new();


//house base 
    house_structure(&mut objects, rock_material.clone());
    house_roof(&mut objects, log_material.clone());
    house_roof_peak(&mut objects, log_material.clone());
    house_peak(&mut objects, log_material.clone());
    cave(
        &mut objects,
        grass_material.clone(),
        soil_material.clone(),
        rock_material.clone(),
        ice_material.clone(),
        lava_material.clone(),
        diamond_material.clone(),
        snow_material.clone(),
    );
    portal_structure(&mut objects, obs_material.clone(), snow_material.clone(), portal_material.clone());
    tree_structure(&mut objects, leaf_material.clone(), log2_material.clone());
    farm(&mut objects, bamboo_material.clone(), soil_material.clone(), water_material.clone(), bush_material.clone(), snow_material.clone(), face_material.clone());

    // Extraemos los bloques emisivos para acelerar el cálculo de luz secundaria
    let emissive_sources: Vec<EmissiveSource> = objects
        .iter()
        .filter(|cube| cube.material.emission_strength > 0.0)
        .map(|cube| {
            let position = cube.center.to_vector3();
            let color = cube.material.emission;
            let strength = cube.material.emission_strength * 1.1;
            let radius = (cube.half.length() * 9.0).max(5.0);

            EmissiveSource {
                position,
                color,
                strength,
                radius,
            }
        })
        .collect();


    // Configuración inicial de la cámara orbital
    let mut camera = Camera::new(
        Vector3::new(0.0, 3.0, 12.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );
    let rotation_speed = PI / 100.0;
    let zoom_speed = 0.1;

    let light = Light::new(
        Vec3::new(8.0, 22.0, 20.0),  
        Color::new(255, 255, 255, 255),
        3.0,  
    );

    // Bucle principal: gestiona entrada, render y presentación en pantalla
    while !window.window_should_close() {
        
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
        
       
        let mut requested_quality_key: Option<&'static str> = None;

        if window.is_key_pressed(KeyboardKey::KEY_ONE) {
            requested_quality_key = Some("potato");
        } else if window.is_key_pressed(KeyboardKey::KEY_TWO) {
            requested_quality_key = Some("low");
        } else if window.is_key_pressed(KeyboardKey::KEY_THREE) {
            requested_quality_key = Some("medium");
        } else if window.is_key_pressed(KeyboardKey::KEY_FOUR) {
            requested_quality_key = Some("high");
        } else if window.is_key_pressed(KeyboardKey::KEY_FIVE) {
            requested_quality_key = Some("ultra");
        }

        if let Some(new_key) = requested_quality_key {
            if new_key != current_framebuffer_key {
                current_framebuffer_key = new_key;
                current_quality = *quality_lookup
                    .get(current_framebuffer_key)
                    .expect("Calidad no configurada");
                println!(
                    "Calidad activa: {} - Cambio instantáneo!",
                    quality_label_for_key(current_framebuffer_key)
                );
            }
        }

        // Obtener el framebuffer correspondiente a la calidad actual
        let current_framebuffer = framebuffers.get_mut(current_framebuffer_key).unwrap();
        
        render(
            current_framebuffer,
            &objects,
            &emissive_sources,
            &camera,
            &light,
            &texture_manager,
            &current_quality,
        );
        
    
        current_framebuffer.swap_buffers(&mut window, &thread, true, window_width, window_height);
    }
}
