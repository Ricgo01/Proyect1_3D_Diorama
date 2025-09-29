use crate::cube::{Cube, Vec3};
use crate::material::Material;

pub fn cave (
    objects: &mut Vec<Cube>,
    grass_material: Material,
    soil_material: Material,
    rock_material: Material,
    ice_material: Material,
    lava_material: Material,
    diamond_material: Material,
    snow_material: Material,
) {


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

        let base_y = 11.0;
        let mut level = 0;
        loop {
            let current_y = base_y + level as f32;
            let x_start = -6 + level;
            let x_end = 6 - level;
            if x_start > x_end {
                break;
            }

            let z_ranges: &[(i32, i32)] = if level < 2 {
                &[(-7, -6)]
            } else {
                &[(-7, -7)]
            };

            for &(z_start, z_end) in z_ranges {
                for x in x_start..=x_end {
                    for z in z_start..=z_end {
                        objects.push(Cube::new(
                            Vec3::new(x as f32, current_y, z as f32),
                            Vec3::new(0.5, 0.5, 0.5),
                            snow_material.clone(),
                        ));
                    }
                }
            }

            level += 1;
            if level >= 6 {
                break;
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



    let y = 9.0;
    for x in -6..=6 {
        for z in -6..=6 {
            objects.push(Cube::new(
                Vec3::new(x as f32, y, z as f32),
                Vec3::new(0.5, 0.5, 0.5),
                diamond_material.clone(),
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