use std::ops::Neg;

#[cfg(not(feature = "simd"))]
mod generic;
#[cfg(feature = "simd")]
mod simd;

pub type V2 = types::V2;
pub type V3 = types::V3;
pub type V4 = types::V4;
pub type M4 = types::M4;
pub type F = f32;
const PI: F = std::f32::consts::PI;

#[cfg(not(feature = "simd"))]
mod types {
    use super::{generic, F};
    pub type V2 = generic::V2<F>;
    pub type V3 = generic::V3<F>;
    pub type V4 = generic::V4<F>;
    pub type M4 = generic::M4<F>;
}

#[cfg(feature = "simd")]
mod types {
    use super::simd;
    pub type V2 = simd::V2;
    pub type V3 = simd::V3;
    pub type V4 = simd::V4;
    pub type M4 = simd::M4;
}

impl V2 {
    pub fn zero() -> Self {
        Self::fill(0.0)
    }

    pub fn one() -> Self {
        Self::fill(1.0)
    }

    pub fn expand(self, z: F) -> V3 {
        V3::new(self.x(), self.y(), z)
    }
}

impl V3 {
    pub fn zero() -> Self {
        Self::fill(0.0)
    }

    pub fn one() -> Self {
        Self::fill(1.0)
    }

    pub fn rand() -> Self {
        Self::new(F::rand(), F::rand(), F::rand())
    }

    pub fn expand(self, w: F) -> V4 {
        V4::new(self.x(), self.y(), self.z(), w)
    }

    pub fn contract(self) -> V2 {
        V2::new(self.x(), self.y())
    }

    pub fn length_squared(&self) -> F {
        self.dot(*self)
    }

    pub fn length(&self) -> F {
        self.length_squared().sqrt()
    }

    pub fn unit(&self) -> Self {
        *self / self.length()
    }

    pub fn random_in_unit_sphere() -> Self {
        loop {
            let x = F::rand() * 2.0 - 1.0;
            let y = F::rand() * 2.0 - 1.0;
            let z = F::rand() * 2.0 - 1.0;

            let v = V3::new(x, y, z);
            if v.length_squared() >= 1.0 {
                continue;
            }
            return v;
        }
    }

    pub fn random_in_unit_disk() -> Self {
        loop {
            let x = F::rand() * 2.0 - 1.0;
            let y = F::rand() * 2.0 - 1.0;

            let v = V3::new(x, y, 0.0);
            if v.length_squared() >= 1.0 {
                continue;
            }
            return v;
        }
    }

    pub fn random_unit_vector() -> Self {
        Self::random_in_unit_sphere().unit()
    }

    pub fn near_zero(&self) -> bool {
        self.x().abs() <= 0.00001 && self.y().abs() <= 0.00001 && self.z().abs() <= 0.00001
    }

    pub fn reflect(&self, normal: Self) -> Self {
        *self - (normal * self.dot(normal) * 2.0)
    }

    pub fn refract(&self, normal: Self, etai_over_etat: F) -> Self {
        let cos_theta = self.neg().dot(normal).min(1.0);
        let r_out_perp = (*self + normal * cos_theta) * etai_over_etat;
        let r_out_parallel = normal * (1.0 - r_out_perp.length_squared()).abs().sqrt().neg();
        r_out_perp + r_out_parallel
    }

    pub fn hsl_to_rgb(&self) -> Self {
        let h = self.x().min(1.0).max(0.0) * 360.0;
        let s = self.y().min(1.0).max(0.0);
        let l = self.z().min(1.0).max(0.0);

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
        match h_prime {
            v if 0.0 <= v && v <= 1.0 => Self::new(c, x, 0.0),
            v if 1.0 <= v && v <= 2.0 => Self::new(x, c, 0.0),
            v if 2.0 <= v && v <= 3.0 => Self::new(0.0, c, x),
            v if 3.0 <= v && v <= 4.0 => Self::new(0.0, x, c),
            v if 4.0 <= v && v <= 5.0 => Self::new(x, 0.0, c),
            v if 5.0 <= v && v <= 6.0 => Self::new(c, 0.0, x),
            _ => Self::fill(0.0),
        }
    }

    pub fn distance(&self, other: Self) -> F {
        let v = *self - other;
        v.dot(v).sqrt()
    }
}

impl V4 {
    pub fn zero() -> Self {
        Self::fill(0.0)
    }

    pub fn one() -> Self {
        Self::fill(1.0)
    }

    pub fn contract(self) -> V3 {
        V3::new(self.x(), self.y(), self.z())
    }
}

impl M4 {
    pub fn identity() -> Self {
        Self::new(
            V4::new(1.0, 0.0, 0.0, 0.0),
            V4::new(0.0, 1.0, 0.0, 0.0),
            V4::new(0.0, 0.0, 1.0, 0.0),
            V4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn translation(translate: V3) -> Self {
        M4::new(
            V4::new(1.0, 0.0, 0.0, 0.0),
            V4::new(0.0, 1.0, 0.0, 0.0),
            V4::new(0.0, 0.0, 1.0, 0.0),
            V4::new(translate.x(), translate.y(), translate.z(), 1.0),
        )
    }

    pub fn rotate_x(angle: F) -> Self {
        let (sin_x, cos_x) = (angle * PI * 2.0).sin_cos();

        M4::new(
            V4::new(1.0, 0.0, 0.0, 0.0),
            V4::new(0.0, cos_x, sin_x, 0.0),
            V4::new(0.0, -sin_x, cos_x, 0.0),
            V4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn rotate_y(angle: F) -> Self {
        let (sin_y, cos_y) = (angle * PI * 2.0).sin_cos();

        M4::new(
            V4::new(cos_y, 0.0, sin_y, 0.0),
            V4::new(0.0, 1.0, 0.0, 0.0),
            V4::new(-sin_y, 0.0, cos_y, 0.0),
            V4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn rotate_z(angle: F) -> Self {
        let (sin_z, cos_z) = (angle * PI * 2.0).sin_cos();

        M4::new(
            V4::new(cos_z, -sin_z, 0.0, 0.0),
            V4::new(sin_z, cos_z, 0.0, 0.0),
            V4::new(0.0, 0.0, 1.0, 0.0),
            V4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn scale(scale: V3) -> Self {
        M4::new(
            V4::new(scale.x(), 0.0, 0.0, 0.0),
            V4::new(0.0, scale.y(), 0.0, 0.0),
            V4::new(0.0, 0.0, scale.z(), 0.0),
            V4::new(0.0, 0.0, 0.0, 1.0),
        )
    }
}

pub trait Num {
    const ZERO: Self;
    const ONE: Self;
    fn min(&self, other: Self) -> Self;
    fn max(&self, other: Self) -> Self;
    fn sqrt(&self) -> Self;
    fn rand() -> Self;
}

impl Num for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;

    fn sqrt(&self) -> Self {
        f32::sqrt(*self)
    }

    fn rand() -> Self {
        fastrand::f32()
    }

    fn min(&self, other: Self) -> Self {
        f32::min(*self, other)
    }

    fn max(&self, other: Self) -> Self {
        f32::max(*self, other)
    }
}

impl Num for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;

    fn sqrt(&self) -> Self {
        f64::sqrt(*self)
    }

    fn rand() -> Self {
        fastrand::f64()
    }

    fn min(&self, other: Self) -> Self {
        f64::min(*self, other)
    }

    fn max(&self, other: Self) -> Self {
        f64::max(*self, other)
    }
}
