use crate::cube::{Cube, Vec3};
use crate::material::Material;

pub fn house_structure(objects: &mut Vec<Cube>, wood_material: Material) {
    let y = -1.0;
    for x in -2 ..=2 {
        for z in -2..=2 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                wood_material.clone(),
            ))
        }
    }
}

pub fn house_walls(objects: &mut Vec<Cube>, wood_material: Material){
    let y = 0.0;
    for x in -2 ..=2 {
        for z in -2..=2 {
            if x == 2 || z == 2 {
                objects.push(Cube::new(
                    Vec3::new(x as f32, y, z as f32),
                    Vec3::new(0.5, 1.5, 0.5),
                    wood_material.clone(),
                ))
            }
        }
    }
}