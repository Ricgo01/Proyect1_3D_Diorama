use crate::cube::{Cube, Vec3};
use crate::material::Material;

pub fn house_structure(objects: &mut Vec<Cube>, wood_material: Material) {
    
    for y in -1..=0 {
        for x in -1 ..=1 {
            for z in -1..=1 {
                objects.push(Cube::new(
                    Vec3::new(x as f32, y as f32, z as f32),
                    Vec3::new(0.5, 0.5, 0.5),
                    wood_material.clone(),
                ))
            }
        }
    }
}


pub fn house_roof(objects: &mut Vec<Cube>, wood_material: Material) {
    let y = 0.0;
    for x in [-2, 2]{
        for z in -1..=1 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                wood_material.clone(),
            ));
        }
    }
}

pub fn house_roof_peak(objects: &mut Vec<Cube>, wood_material: Material) {
    let y = 1.0;
    for x in -1..=1 {
        for z in -1..=1 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                wood_material.clone(),
            ));
        }
    }
}

pub fn house_peak(objects: &mut Vec<Cube>, wood_material: Material) {
    let y = 2.0;
    let x = 0.0;
    for z in -1..=1 {
        objects.push(Cube::new(
            Vec3::new(x as f32, y as f32, z as f32),
            Vec3::new(0.5, 0.5, 0.5),
            wood_material.clone(),
        ))
    }
}