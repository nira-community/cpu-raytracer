use rand::Rng;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::hittable::Hittable;
use crate::ray::Ray;
use crate::vec3::{cross, random_in_unit_disk, Color, Point3, Vec3};

pub struct CameraConfig {
    pub aspect_ratio: f64,
    pub image_width: usize,
    pub samples_per_pixel: usize,
    pub max_depth: i32,

    pub vfov: f64,
    pub lookfrom: Point3,
    pub lookat: Point3,
    pub vup: Vec3,

    pub defocus_angle: f64,
    pub focus_dist: f64,
}

impl Default for CameraConfig {
    fn default() -> Self {
        CameraConfig {
            aspect_ratio: 16.0 / 9.0,
            image_width: 400,
            samples_per_pixel: 50,
            max_depth: 50,
            vfov: 90.0,
            lookfrom: Point3::new(0.0, 0.0, 0.0),
            lookat: Point3::new(0.0, 0.0, -1.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle: 0.0,
            focus_dist: 10.0,
        }
    }
}

pub struct Camera {
    cfg: CameraConfig,
    image_height: usize,
    center: Point3,
    pixel00: Point3,
    du: Vec3,
    dv: Vec3,
    defocus_u: Vec3,
    defocus_v: Vec3,
}

impl Camera {
    pub fn new(cfg: CameraConfig) -> Camera {
        let image_height = ((cfg.image_width as f64 / cfg.aspect_ratio) as usize).max(1);
        let center = cfg.lookfrom;

        let theta = cfg.vfov.to_radians();
        let h = (theta / 2.0).tan();
        let vp_h = 2.0 * h * cfg.focus_dist;
        let vp_w = vp_h * (cfg.image_width as f64 / image_height as f64);

        let w = (cfg.lookfrom - cfg.lookat).unit();
        let u = cross(cfg.vup, w).unit();
        let v = cross(w, u);

        let vp_u = vp_w * u;
        let vp_v = -vp_h * v;

        let du = vp_u / cfg.image_width as f64;
        let dv = vp_v / image_height as f64;

        let vp_upper_left = center - cfg.focus_dist * w - vp_u / 2.0 - vp_v / 2.0;
        let pixel00 = vp_upper_left + 0.5 * (du + dv);

        let defocus_radius = cfg.focus_dist * (cfg.defocus_angle / 2.0).to_radians().tan();

        Camera {
            image_height,
            center,
            pixel00,
            du,
            dv,
            defocus_u: u * defocus_radius,
            defocus_v: v * defocus_radius,
            cfg,
        }
    }

    pub fn render(&self, world: &dyn Hittable, path: &str) -> std::io::Result<()> {
        let w = self.cfg.image_width;
        let h = self.image_height;
        let done = AtomicUsize::new(0);

        let rows: Vec<Vec<Color>> = (0..h)
            .into_par_iter()
            .map(|j| {
                let row = (0..w)
                    .map(|i| {
                        let mut c = Color::new(0.0, 0.0, 0.0);
                        for _ in 0..self.cfg.samples_per_pixel {
                            let r = self.get_ray(i, j);
                            c += ray_color(&r, world, self.cfg.max_depth);
                        }
                        c / self.cfg.samples_per_pixel as f64
                    })
                    .collect();

                let d = done.fetch_add(1, Ordering::Relaxed) + 1;
                if d % 20 == 0 || d == h {
                    eprint!("\rscanlines done: {}/{}  ", d, h);
                }
                row
            })
            .collect();
        eprintln!();

        let mut out = BufWriter::new(File::create(path)?);
        writeln!(out, "P3\n{} {}\n255", w, h)?;
        for row in &rows {
            for px in row {
                write_color(&mut out, *px)?;
            }
        }
        out.flush()?;
        Ok(())
    }

    fn get_ray(&self, i: usize, j: usize) -> Ray {
        let mut rng = rand::thread_rng();
        let ox: f64 = rng.gen::<f64>() - 0.5;
        let oy: f64 = rng.gen::<f64>() - 0.5;
        let sample = self.pixel00 + (i as f64 + ox) * self.du + (j as f64 + oy) * self.dv;

        let orig = if self.cfg.defocus_angle <= 0.0 {
            self.center
        } else {
            let p = random_in_unit_disk();
            self.center + p.x * self.defocus_u + p.y * self.defocus_v
        };

        Ray::new(orig, sample - orig)
    }
}

fn ray_color(r: &Ray, world: &dyn Hittable, depth: i32) -> Color {
    if depth <= 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    if let Some(rec) = world.hit(r, 0.001, f64::INFINITY) {
        return match rec.mat.scatter(r, &rec) {
            Some((att, scattered)) => att * ray_color(&scattered, world, depth - 1),
            None => Color::new(0.0, 0.0, 0.0),
        };
    }

    let unit = r.dir.unit();
    let a = 0.5 * (unit.y + 1.0);
    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}

fn write_color<W: Write>(out: &mut W, c: Color) -> std::io::Result<()> {
    // gamma 2
    let f = |x: f64| (256.0 * x.sqrt().clamp(0.0, 0.999)) as u32;
    writeln!(out, "{} {} {}", f(c.x), f(c.y), f(c.z))
}
