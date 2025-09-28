use crate::cube::{Cube, Vec3};
use crate::material::Material;

pub fn portal_structure(objects: &mut Vec<Cube>, obs_material: Material, snow_material: Material, portal_material: Material) {
    let y = 11.0;
    for x in [-5, 5]{
        for z in -1..=1 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                obs_material.clone(),
            ));
        }
    }

    let y = 11.0;
    for z in [-5, 5]{
        for x in -1..=1 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                obs_material.clone(),
            ));
        }
    }

    let y = 12.0;
    for z in [-4, 4]{
        for x in -1..=1 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                obs_material.clone(),
            ));
        }
    }

    let y = 12.0;
    for x in [-4, 4]{
        for z in -1..=1 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                obs_material.clone(),
            ));
        }
    }

    let y = 12.0;
    for x in -3..=3 {
        for z in -3..=3 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                snow_material.clone(),
            ));
        }
    }

    let z = 0.0;
    for x in [-2, 2]{
        for y in 13..=17 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y as f32, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                obs_material.clone(),
            ));
        }
    }

    let z = 0.0;
    for y in [13.0, 17.0]{
        for x in -2..=2 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y as f32, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                obs_material.clone(),
            ));
        }
    }

    let z = 0.0;
    for y in 12..=16 {
        for x in -1..=1 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y as f32, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                portal_material.clone(),
            ));
        }
    }


}