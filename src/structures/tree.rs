use crate::cube::{Cube, Vec3};
use crate::material::Material;

pub fn tree_structure(objects: &mut Vec<Cube>, leaf_material: Material, log_material: Material) {
    // Tronco

    let x = -5.0;
    let z = -4.0;
    for y in 0..3 {
        objects.push(Cube::new(
            Vec3::new(x, y as f32, z),
            Vec3::new(0.5, 0.5, 0.5),
            log_material.clone(),
        ));
    }

    //hojas - forma de cruz más eficiente (solo 13 bloques vs 27)
    let trunk_x = -5.0;
    let trunk_z = -4.0;
    
    // Nivel superior (y=5) - solo centro
    objects.push(Cube::new(
        Vec3::new(trunk_x, 5.0, trunk_z),
        Vec3::new(0.5, 0.5, 0.5),
        leaf_material.clone(),
    ));
    
    // Nivel medio (y=4) - forma de cruz
    for (dx, dz) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] {
        objects.push(Cube::new(
            Vec3::new(trunk_x + dx as f32, 4.0, trunk_z + dz as f32),
            Vec3::new(0.5, 0.5, 0.5),
            leaf_material.clone(),
        ));
    }
    
    // Nivel inferior (y=3) - forma de cruz extendida
    for (dx, dz) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, 1)] {
        objects.push(Cube::new(
            Vec3::new(trunk_x + dx as f32, 3.0, trunk_z + dz as f32),
            Vec3::new(0.5, 0.5, 0.5),
            leaf_material.clone(),
        ));
    }
}