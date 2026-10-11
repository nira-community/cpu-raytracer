mod camera;
mod hittable;
mod material;
mod ray;
mod sphere;
mod vec3;

use rand::Rng;
use std::time::Instant;

use camera::{Camera, CameraConfig};
use hittable::HittableList;
use material::Material;
use sphere::Sphere;
use vec3::{Color, Point3, Vec3};

fn random_scene() -> HittableList {
    let mut world = HittableList::new();
    let mut rng = rand::thread_rng();

    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Material::Lambertian(Color::new(0.5, 0.5, 0.5)),
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose: f64 = rng.gen();
            let center = Point3::new(
                a as f64 + 0.9 * rng.gen::<f64>(),
                0.2,
                b as f64 + 0.9 * rng.gen::<f64>(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() <= 0.9 {
                continue;
            }

            let mat = if choose < 0.8 {
                let albedo = Color::random() * Color::random();
                Material::Lambertian(albedo)
            } else if choose < 0.95 {
                let albedo = Color::random_range(0.5, 1.0);
                let fuzz = rng.gen_range(0.0..0.5);
                Material::Metal(albedo, fuzz)
            } else {
                Material::Dielectric(1.5)
            };
            world.add(Box::new(Sphere::new(center, 0.2, mat)));
        }
    }

    world.add(Box::new(Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, Material::Dielectric(1.5))));
    world.add(Box::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        Material::Lambertian(Color::new(0.4, 0.2, 0.1)),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        Material::Metal(Color::new(0.7, 0.6, 0.5), 0.0),
    )));

    world
}

// small scene i used for testing refraction / fov
#[allow(dead_code)]
fn test_scene() -> HittableList {
    let mut world = HittableList::new();
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -100.5, -1.0),
        100.0,
        Material::Lambertian(Color::new(0.8, 0.8, 0.0)),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 0.0, -1.2),
        0.5,
        Material::Lambertian(Color::new(0.1, 0.2, 0.5)),
    )));
    world.add(Box::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, Material::Dielectric(1.5))));
    world.add(Box::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.4, Material::Dielectric(1.0 / 1.5))));
    world.add(Box::new(Sphere::new(
        Point3::new(1.0, 0.0, -1.0),
        0.5,
        Material::Metal(Color::new(0.8, 0.6, 0.2), 1.0),
    )));
    world
}

fn main() {
    // usage: raytracer [width] [samples] [out]
    let args: Vec<String> = std::env::args().collect();
    let width = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(400);
    let samples = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(50);
    let out = args.get(3).map(|s| s.as_str()).unwrap_or("output/image.ppm");

    let world = random_scene();
    //let world = test_scene();

    let cam = Camera::new(CameraConfig {
        aspect_ratio: 16.0 / 9.0,
        image_width: width,
        samples_per_pixel: samples,
        max_depth: 50,
        vfov: 20.0,
        lookfrom: Point3::new(13.0, 2.0, 3.0),
        lookat: Point3::new(0.0, 0.0, 0.0),
        vup: Vec3::new(0.0, 1.0, 0.0),
        defocus_angle: 0.6,
        focus_dist: 10.0,
    });

    if let Some(dir) = std::path::Path::new(out).parent() {
        std::fs::create_dir_all(dir).unwrap();
    }

    let start = Instant::now();
    cam.render(&world, out).expect("failed to write image");
    eprintln!("done in {:.2?} -> {}", start.elapsed(), out);
}
