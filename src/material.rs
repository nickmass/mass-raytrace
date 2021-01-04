use std::ops::Neg;

use super::geom::Hit;
use super::world::Ray;
use crate::math::{Num, V3};

pub struct Scatter {
    pub attenuation: V3,
    pub scattered: Ray,
}

pub trait Material: Send + Sync {
    fn scatter(&self, ray: Ray, hit: &Hit) -> Option<Scatter>;
    fn emit(&self, _hit: &Hit) -> Option<V3> {
        None
    }
}

pub trait Background: Send + Sync {
    fn background(&self, ray: Ray) -> V3;
}

pub struct SolidBackground {
    color: V3,
}

impl SolidBackground {
    pub fn new(color: V3) -> Self {
        Self { color }
    }
}

impl Background for SolidBackground {
    fn background(&self, _ray: Ray) -> V3 {
        self.color
    }
}

pub struct SkyBackground;

impl Background for SkyBackground {
    fn background(&self, ray: Ray) -> V3 {
        let unit_direction = ray.direction.unit();
        let t = 0.5 * (unit_direction.y() + 1.0);
        (V3::fill(1.0) * (1.0 - t)) + (V3::new(0.5, 0.7, 1.0) * t)
    }
}

#[derive(Copy, Clone)]
pub struct Lambertian {
    albedo: V3,
}

impl Lambertian {
    pub fn new(albedo: V3) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: Ray, hit: &Hit) -> Option<Scatter> {
        let scatter_direction = hit.normal + V3::random_unit_vector();
        let scatter_direction = if scatter_direction.near_zero() {
            hit.normal
        } else {
            scatter_direction
        };
        let scattered = Ray::new(hit.point, scatter_direction);

        Some(Scatter {
            scattered,
            attenuation: self.albedo,
        })
    }
}

#[derive(Copy, Clone)]
pub struct DiffuseLight {
    emit: V3,
}

impl DiffuseLight {
    pub fn new(emit: V3) -> Self {
        Self { emit }
    }
}

impl Material for DiffuseLight {
    fn scatter(&self, _ray: Ray, _hit: &Hit) -> Option<Scatter> {
        None
    }

    fn emit(&self, _hit: &Hit) -> Option<V3> {
        Some(self.emit)
    }
}

#[derive(Copy, Clone)]
pub struct Metal {
    fuzz: f64,
    albedo: V3,
}

impl Metal {
    pub fn new(fuzz: f64, albedo: V3) -> Self {
        let fuzz = if fuzz < 1.0 { fuzz } else { 1.0 };
        Self { fuzz, albedo }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: Ray, hit: &Hit) -> Option<Scatter> {
        let reflected = ray.direction.unit().reflect(hit.normal);
        let scattered = Ray::new(
            hit.point,
            reflected + (V3::random_in_unit_sphere() * self.fuzz),
        );

        if scattered.direction.dot(&hit.normal) > 0.0 {
            Some(Scatter {
                scattered,
                attenuation: self.albedo,
            })
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Dielectric {
    refraction_index: f64,
}

impl Dielectric {
    pub fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
    }

    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: Ray, hit: &Hit) -> Option<Scatter> {
        let attenuation = V3::fill(1.0);
        let refraction_ratio = if hit.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = ray.direction.unit();
        let cos_theta = unit_direction.neg().dot(&hit.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let direction =
            if cannot_refract || Self::reflectance(cos_theta, refraction_ratio) > f64::rand() {
                unit_direction.reflect(hit.normal)
            } else {
                unit_direction.refract(hit.normal, refraction_ratio)
            };

        Some(Scatter {
            attenuation,
            scattered: Ray::new(hit.point, direction),
        })
    }
}

#[derive(Copy, Clone)]
pub struct Specular {
    refraction_index: f64,
    inner: Lambertian,
}

impl Specular {
    pub fn new(refraction_index: f64, albedo: V3) -> Self {
        let mat = Lambertian::new(albedo);
        Self {
            refraction_index,
            inner: mat,
        }
    }

    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Specular {
    fn scatter(&self, ray: Ray, hit: &Hit) -> Option<Scatter> {
        let attenuation = V3::fill(1.0);
        let refraction_ratio = if hit.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = ray.direction.unit();
        let cos_theta = unit_direction.neg().dot(&hit.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let direction =
            if cannot_refract || Self::reflectance(cos_theta, refraction_ratio) > f64::rand() {
                unit_direction.reflect(hit.normal)
            } else {
                return self.inner.scatter(ray, hit);
            };

        Some(Scatter {
            attenuation,
            scattered: Ray::new(hit.point, direction),
        })
    }
}

impl Material for () {
    fn scatter(&self, _ray: Ray, _hit: &Hit) -> Option<Scatter> {
        None
    }
}
