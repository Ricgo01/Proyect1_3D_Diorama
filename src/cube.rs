use raylib::prelude::Vector3;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::material::Material;
use std::ops::{Add, Sub, Mul, Div, Neg};

#[derive(Clone, Copy, Debug, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vec3 {
    #[inline] pub const fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    #[inline] pub fn dot(self, o: Self) -> f32 { self.x * o.x + self.y * o.y + self.z * o.z }
    #[inline] pub fn length(self) -> f32 { self.dot(self).sqrt() }
    #[inline] pub fn normalized(self) -> Self {
        let l = self.length();
        if l > 0.0 { self / l } else { self }
    }
    #[inline] pub fn normalize(self) -> Self {
        self.normalized()
    }
    #[inline] pub fn cross(self, o: Self) -> Self {
        Self::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    #[inline]
    pub fn from_vector3(v: Vector3) -> Self {
        Self::new(v.x, v.y, v.z)
    }

    #[inline]
    pub fn to_vector3(self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }
}

impl Add for Vec3 { type Output = Self; fn add(self, o: Self) -> Self { Self::new(self.x+o.x, self.y+o.y, self.z+o.z) } }
impl Sub for Vec3 { type Output = Self; fn sub(self, o: Self) -> Self { Self::new(self.x-o.x, self.y-o.y, self.z-o.z) } }
impl Mul<f32> for Vec3 { type Output = Self; fn mul(self, s: f32) -> Self { Self::new(self.x*s, self.y*s, self.z*s) } }
impl Mul for Vec3 { type Output = Self; fn mul(self, o: Self) -> Self { Self::new(self.x*o.x, self.y*o.y, self.z*o.z) } }
impl Div<f32> for Vec3 { type Output = Self; fn div(self, s: f32) -> Self { Self::new(self.x/s, self.y/s, self.z/s) } }
impl Neg for Vec3 { type Output = Self; fn neg(self) -> Self { Self::new(-self.x, -self.y, -self.z) } }

#[derive(Clone, Debug)]
pub struct Cube {
    pub center: Vec3,
    pub half: Vec3,
    pub material: Material,
}

impl Cube {
    pub fn new(center: Vec3, half: Vec3, material: Material) -> Self { 
        Self { center, half, material } 
    }

    #[inline]
    pub fn intersect(&self, ro: Vec3, rd: Vec3) -> Option<(f32, Vec3, f32, f32)> {
        let minb = self.center - self.half;
        let maxb = self.center + self.half;

        let inv_rd = Vec3::new(
            if rd.x.abs() > 1e-8 { 1.0 / rd.x } else { f32::INFINITY },
            if rd.y.abs() > 1e-8 { 1.0 / rd.y } else { f32::INFINITY },
            if rd.z.abs() > 1e-8 { 1.0 / rd.z } else { f32::INFINITY },
        );

        let t1 = (minb - ro) * inv_rd;
        let t2 = (maxb - ro) * inv_rd;

        let t_min = Vec3::new(t1.x.min(t2.x), t1.y.min(t2.y), t1.z.min(t2.z));
        let t_max = Vec3::new(t1.x.max(t2.x), t1.y.max(t2.y), t1.z.max(t2.z));

        let t_near = t_min.x.max(t_min.y).max(t_min.z);
        let t_far = t_max.x.min(t_max.y).min(t_max.z);

        if t_near > t_far || t_far < 0.0 {
            return None;
        }

        let t_hit = if t_near >= 0.0 { t_near } else { t_far };
        let p = ro + rd * t_hit;

        let local = p - self.center;
        let dx = (local.x.abs() - self.half.x).abs();
        let dy = (local.y.abs() - self.half.y).abs();
        let dz = (local.z.abs() - self.half.z).abs();
        let eps = 1e-3;

        let (n, u, v) = if dx <= dy && dx <= dz && dx < eps {
            let normal = Vec3::new(local.x.signum(), 0.0, 0.0);
            
            let u = (local.z / self.half.z + 1.0) * 0.5;
            let v = (local.y / self.half.y + 1.0) * 0.5;
            
            (normal, u.clamp(0.0, 1.0), 1.0 - v.clamp(0.0, 1.0))
        } else if dy <= dx && dy <= dz && dy < eps {
            
            let normal = Vec3::new(0.0, local.y.signum(), 0.0);
            
        
            let u = (local.x / self.half.x + 1.0) * 0.5;
            let v = (local.z / self.half.z + 1.0) * 0.5;
            
            (normal, u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
        } else {
           
            let normal = Vec3::new(0.0, 0.0, local.z.signum());
            
           
            let u = if local.z > 0.0 {
                (local.x / self.half.x + 1.0) * 0.5
            } else {
                (-local.x / self.half.x + 1.0) * 0.5
            };
            let v = (local.y / self.half.y + 1.0) * 0.5;
            
            (normal, u.clamp(0.0, 1.0), 1.0 - v.clamp(0.0, 1.0))
        };

        Some((t_hit, n, u, v))
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect {
        let ro = Vec3::from_vector3(*ray_origin);
        let rd = Vec3::from_vector3(*ray_direction);
        
        if let Some((t, normal, u, v)) = self.intersect(ro, rd) {
            let point = ro + rd * t;
            Intersect::new(
                point.to_vector3(),
                normal.to_vector3(),
                t,
                self.material.clone(),
                u,
                v,
            )
        } else {
            Intersect::empty()
        }
    }
}
