use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{dot, Point3};

pub struct Sphere {
    center: Point3,
    radius: f64,
    mat: Material,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat: Material) -> Sphere {
        Sphere { center, radius, mat }
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = r.orig - self.center;
        let a = r.dir.length_squared();
        let half_b = dot(oc, r.dir);
        let c = oc.length_squared() - self.radius * self.radius;

        let disc = half_b * half_b - a * c;
        if disc < 0.0 {
            return None;
        }
        let sq = disc.sqrt();

        let mut root = (-half_b - sq) / a;
        if root <= t_min || root >= t_max {
            root = (-half_b + sq) / a;
            if root <= t_min || root >= t_max {
                return None;
            }
        }

        let p = r.at(root);
        let outward = (p - self.center) / self.radius;
        Some(HitRecord::new(r, root, p, outward, &self.mat))
    }
}
