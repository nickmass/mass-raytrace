use std::ops::Neg;

use super::geom::Hit;
use super::world::Ray;
use crate::{
    math::{Num, M4, V2, V3},
    texture::Surface,
};

pub struct Scatter {
    pub attenuation: V3,
    pub scattered: Ray,
}

pub trait Material: Send + Sync {
    fn scatter(&self, ray: Ray, hit: &Hit) -> Option<Scatter>;
    fn emit(&self, _hit: &Hit) -> Option<V3> {
        None
    }
    fn normal(&self, _uv: V2) -> Option<V3> {
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

pub struct SkySphere<S: Surface> {
    texture: S,
}

impl<S: Surface> SkySphere<S> {
    pub fn new(texture: S) -> Self {
        Self { texture }
    }
}

impl<S: Surface> Background for SkySphere<S> {
    fn background(&self, ray: Ray) -> V3 {
        let p = ray.direction.unit();
        let theta = (p.y()).acos();
        let phi = (p.z() * -1.0).atan2(p.x()) + std::f32::consts::PI;

        let uv = V2::new(
            phi / (2.0 * std::f32::consts::PI),
            theta / std::f32::consts::PI,
        );

        let pixel = self.texture.get_f(uv);
        V3::new(pixel.x(), pixel.y(), pixel.z())
    }
}

pub struct CubeMap<S: Surface> {
    x_pos: S,
    x_neg: S,
    y_pos: S,
    y_neg: S,
    z_pos: S,
    z_neg: S,
    transform: M4,
}

impl<S: Surface> CubeMap<S> {
    pub fn new(x_pos: S, x_neg: S, y_pos: S, y_neg: S, z_pos: S, z_neg: S, rotation: V3) -> Self {
        let rotate_x = M4::rotate_x(rotation.x());
        let rotate_y = M4::rotate_x(rotation.y());
        let rotate_z = M4::rotate_x(rotation.z());

        let transform = rotate_x * rotate_y * rotate_z;

        Self {
            x_pos,
            x_neg,
            y_pos,
            y_neg,
            z_pos,
            z_neg,
            transform,
        }
    }
}

impl<S: Surface> Background for CubeMap<S> {
    fn background(&self, ray: Ray) -> V3 {
        let p = self.transform.transform_vector(ray.direction);

        let abs_p = p.abs();

        let is_x_pos = p.x() > 0.0;
        let is_y_pos = p.y() > 0.0;
        let is_z_pos = p.z() > 0.0;

        let is_x_large = abs_p.x() >= abs_p.y() && abs_p.x() >= abs_p.z();
        let is_y_large = abs_p.y() >= abs_p.x() && abs_p.y() >= abs_p.z();
        let is_z_large = abs_p.z() >= abs_p.x() && abs_p.z() >= abs_p.y();

        let mut index = 0;
        let mut max_axis = 0.0;

        let mut u = 0.0;
        let mut v = 0.0;

        if is_x_large {
            if is_x_pos {
                index = 0;
                u = p.z() * -1.0;
                v = p.y();
            } else {
                index = 1;
                u = p.z();
                v = p.y();
            }
            max_axis = abs_p.x();
        } else if is_y_large {
            if is_y_pos {
                index = 3;
                u = p.x();
                v = p.z() * -1.0;
            } else {
                index = 2;
                u = p.x();
                v = p.z();
            }
            max_axis = abs_p.y();
        } else if is_z_large {
            if is_z_pos {
                index = 4;
                u = p.x();
                v = p.y();
            } else {
                index = 5;
                u = p.x() * -1.0;
                v = p.y();
            }
            max_axis = abs_p.z();
        }

        let uv = V2::new(0.5 * (u / max_axis + 1.0), 0.5 * (v / max_axis + 1.0));

        let color = match index {
            0 => self.x_pos.get_f(uv),
            1 => self.x_neg.get_f(uv),
            2 => self.y_pos.get_f(uv),
            3 => self.y_neg.get_f(uv),
            4 => self.z_pos.get_f(uv),
            5 => self.z_neg.get_f(uv),
            _ => unreachable!(),
        };

        color.contract()
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
    fuzz: f32,
    albedo: V3,
}

impl Metal {
    pub fn new(fuzz: f32, albedo: V3) -> Self {
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

        if scattered.direction.dot(hit.normal) > 0.0 {
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
    refraction_index: f32,
}

impl Dielectric {
    pub fn new(refraction_index: f32) -> Self {
        Self { refraction_index }
    }

    fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
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
        let cos_theta = unit_direction.neg().dot(hit.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let direction =
            if cannot_refract || Self::reflectance(cos_theta, refraction_ratio) > f32::rand() {
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
    refraction_index: f32,
    inner: Lambertian,
}

impl Specular {
    pub fn new(refraction_index: f32, albedo: V3) -> Self {
        let mat = Lambertian::new(albedo);
        Self {
            refraction_index,
            inner: mat,
        }
    }

    fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
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
        let cos_theta = unit_direction.neg().dot(hit.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let direction =
            if cannot_refract || Self::reflectance(cos_theta, refraction_ratio) > f32::rand() {
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

pub struct Mix<MLeft: Material, MRight: Material> {
    ratio: f32,
    left: MLeft,
    right: MRight,
}

impl<MLeft: Material, MRight: Material> Mix<MLeft, MRight> {
    pub fn new(ratio: f32, left: MLeft, right: MRight) -> Self {
        Self { ratio, left, right }
    }
}
impl<MLeft: Material, MRight: Material> Material for Mix<MLeft, MRight> {
    fn scatter(&self, ray: Ray, hit: &Hit) -> Option<Scatter> {
        if f32::rand() < self.ratio {
            self.left.scatter(ray, hit)
        } else {
            self.right.scatter(ray, hit)
        }
    }

    fn emit(&self, hit: &Hit) -> Option<V3> {
        if f32::rand() < self.ratio {
            self.left.emit(hit)
        } else {
            self.right.emit(hit)
        }
    }
}
