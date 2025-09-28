use crate::cube::{Cube, Vec3};
use crate::material::Material;

pub fn cave (objects: &mut Vec<Cube>, grass_material: Material, soil_material: Material, rock_material: Material, ice_material: Material, lava_material: Material) {


    let y = -2.0;
    for x in -6..=6 {
        for z in -6..=6 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                grass_material.clone(),
            ));
        }
    }

    for y in -5..=-3{
        for x in -6..=6 {
            for z in -6..=6 {
                objects.push(Cube::new(
                    Vec3::new(x as f32, y as f32, z as f32),
                    Vec3::new(0.5, 0.5, 0.5),
                    soil_material.clone(),
                ));
            }
        }
    }

    for x in [-6]{
        for z in -6..=6 {
            for y in -1..=9 {
                objects.push(Cube::new(
                    Vec3::new(x as f32, y as f32, z as f32),
                    Vec3::new(0.5, 0.5, 0.5),
                    rock_material.clone(),
                ));
            }
        }
    }

    for z in [-6]{
        for x in -6..=6{
            for y in -1..=9{
                objects.push(Cube::new(
                    Vec3::new(x as f32, y as f32, z as f32),
                    Vec3::new(0.5, 0.5, 0.5),
                    rock_material.clone(),
                ));
            }
        }
    }

    let y = 10.0;
    for x in -6..=6 {
        for z in -6..=6 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                ice_material.clone(),
            ));
        }
    }

    let y = 6.0;
    let x = -5.0;
    for z in -0..=1 {
        objects.push(Cube::new(
            Vec3::new(x, y, z as f32),
            Vec3::new(0.5, 0.5, 0.5),
            lava_material.clone(),
        ));
    }

    let y = 5.0;
    let x = -4.0;
    for z in -0..=1 {
        objects.push(Cube::new(
            Vec3::new(x, y, z as f32),
            Vec3::new(0.5, 0.5, 0.5),
            lava_material.clone(),
        ));
    }

    
    let x = -4.0;
    for y in -1..=4 {
        for z in -0..=1 {
            objects.push(Cube::new(
                Vec3::new(x, y as f32, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                lava_material.clone(),
            ));
        }
    }

}