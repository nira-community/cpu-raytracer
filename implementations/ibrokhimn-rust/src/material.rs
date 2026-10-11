use rand::Rng;

use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::vec3::{dot, reflect, refract, random_unit_vector, Color};

#[derive(Clone, Copy, Debug)]
pub enum Material {
    Lambertian(Color),
    Metal(Color, f64),
    Dielectric(f64),
}

impl Material {
    pub fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        match *self {
            Material::Lambertian(albedo) => {
                let mut dir = rec.normal + random_unit_vector();
                if dir.near_zero() {
                    dir = rec.normal;
                }
                Some((albedo, Ray::new(rec.p, dir)))
            }
            Material::Metal(albedo, fuzz) => {
                let refl = reflect(r_in.dir.unit(), rec.normal);
                let scattered = Ray::new(rec.p, refl + fuzz * random_unit_vector());
                if dot(scattered.dir, rec.normal) > 0.0 {
                    Some((albedo, scattered))
                } else {
                    None
                }
            }
            Material::Dielectric(ir) => {
                let ratio = if rec.front_face { 1.0 / ir } else { ir };
                let unit_dir = r_in.dir.unit();

                let cos_theta = dot(-unit_dir, rec.normal).min(1.0);
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

                let cannot_refract = ratio * sin_theta > 1.0;
                let dir = if cannot_refract || reflectance(cos_theta, ratio) > rand::thread_rng().gen::<f64>() {
                    reflect(unit_dir, rec.normal)
                } else {
                    refract(unit_dir, rec.normal, ratio)
                };

                Some((Color::new(1.0, 1.0, 1.0), Ray::new(rec.p, dir)))
            }
        }
    }
}

// schlick
fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
    let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}
