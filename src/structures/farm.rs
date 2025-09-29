use crate::cube::{Cube, Vec3};
use crate::material::Material;

pub fn farm(objects: &mut Vec<Cube>, bamboo_material: Material, soil_material: Material, water_material: Material, bush_material: Material, snow_material: Material, face_material: Material) {

    let y = -1.0;
    for x in [2, 6]{
        for z in -4..=2 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                bamboo_material.clone(),
            ));
        }
    }

    let y = -1.0;
    for z in [-4, 2]{
        for x in 3..=6 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                bamboo_material.clone(),
            ));
        }
    }

    let y = -1.0;
    for x in [3, 5]{
        for z in -3..=1 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                soil_material.clone(),
            ));
        }
    }

    let y = -1.0;
    for x in [4]{
        for z in -3..=1 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                water_material.clone(),
            ));
        }
    }

    let y = -1.0;
    for x in [4, 5]{
        for z in 4..=5 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                bush_material.clone(),
            ));
        }
    }

    let x = -1.0;
    let z = -1.0;
    for y in -1..1 {
        objects.push(Cube::new(
            Vec3::new(x, y as f32, z),
            Vec3::new(0.5, 0.5, 0.5),
            snow_material.clone(),
        ));
    }

    let x = -1.0;
    let z = -1.0;
    for y in [1] {
        objects.push(Cube::new(
            Vec3::new(x, y as f32, z),
            Vec3::new(0.5, 0.5, 0.5),
            face_material.clone(),
        ));
    }
}