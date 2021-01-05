#![allow(dead_code)]
use std::ops::{Add, Div, Mul, Sub};

#[cfg(feature = "simd")]
pub type V3 = simd::V3;
#[cfg(feature = "simd")]
pub type M4 = simd::M4;

#[cfg(not(feature = "simd"))]
pub type V3 = generic::V3<f32>;
#[cfg(not(feature = "simd"))]
pub type M4 = generic::M4<f32>;

pub trait Num:
    Mul<Output = Self> + Add<Output = Self> + Div<Output = Self> + Sub<Output = Self> + Copy
{
    const ZERO: Self;
    fn sqrt(&self) -> Self;
    fn rand() -> Self;
}

impl Num for f32 {
    const ZERO: Self = 0.0;

    fn sqrt(&self) -> Self {
        f32::sqrt(*self)
    }

    fn rand() -> Self {
        fastrand::f32()
    }
}

impl Num for f64 {
    const ZERO: Self = 0.0;

    fn sqrt(&self) -> Self {
        f64::sqrt(*self)
    }

    fn rand() -> Self {
        fastrand::f64()
    }
}

#[cfg(not(feature = "simd"))]
pub mod generic {
    use super::Num;
    use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

    #[derive(Copy, Clone, Debug)]
    pub struct V3<T> {
        x: T,
        y: T,
        z: T,
    }

    impl<T> V3<T> {
        pub fn new(x: T, y: T, z: T) -> Self {
            Self { x, y, z }
        }
    }

    impl<T: Copy> V3<T> {
        pub fn x(&self) -> T {
            self.x
        }

        pub fn y(&self) -> T {
            self.y
        }

        pub fn z(&self) -> T {
            self.z
        }
    }

    impl<T: Clone> V3<T> {
        pub fn fill(value: T) -> Self {
            Self {
                x: value.clone(),
                y: value.clone(),
                z: value.clone(),
            }
        }
    }

    impl<T: Num> V3<T> {
        pub fn zero() -> Self {
            Self {
                x: T::ZERO,
                y: T::ZERO,
                z: T::ZERO,
            }
        }

        pub fn rand() -> Self {
            Self::new(T::rand(), T::rand(), T::rand())
        }

        pub fn length_squared(&self) -> T {
            (self.x * self.x) + (self.y * self.y) + (self.z * self.z)
        }

        pub fn length(&self) -> T {
            self.length_squared().sqrt()
        }

        pub fn unit(&self) -> Self {
            *self / self.length()
        }

        pub fn dot(&self, other: &Self) -> T {
            self.x * other.x + self.y * other.y + self.z * other.z
        }

        pub fn cross(&self, other: &Self) -> Self {
            Self::new(
                self.y * other.z - self.z * other.y,
                self.z * other.x - self.x * other.z,
                self.x * other.y - self.y * other.x,
            )
        }
    }

    impl V3<f32> {
        pub fn random_in_unit_sphere() -> Self {
            loop {
                let x = f32::rand() * 2.0 - 1.0;
                let y = f32::rand() * 2.0 - 1.0;
                let z = f32::rand() * 2.0 - 1.0;

                let v = V3::new(x, y, z);
                if v.length_squared() >= 1.0 {
                    continue;
                }
                return v;
            }
        }

        pub fn random_in_unit_disk() -> Self {
            loop {
                let x = f32::rand() * 2.0 - 1.0;
                let y = f32::rand() * 2.0 - 1.0;

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
            self.x.abs() <= 3.0 * f32::EPSILON
                && self.y.abs() <= 3.0 * f32::EPSILON
                && self.z.abs() <= 3.0 * f32::EPSILON
        }

        pub fn reflect(&self, normal: Self) -> Self {
            *self - (normal * self.dot(&normal) * 2.0)
        }

        pub fn refract(&self, normal: Self, etai_over_etat: f32) -> Self {
            let cos_theta = self.neg().dot(&normal).min(1.0);
            let r_out_perp = (*self + normal * cos_theta) * etai_over_etat;
            let r_out_parallel = normal * (1.0 - r_out_perp.length_squared()).abs().sqrt().neg();
            r_out_perp + r_out_parallel
        }

        pub fn hsl_to_rgb(&self) -> Self {
            let h = self.x.min(1.0).max(0.0) * 360.0;
            let s = self.y.min(1.0).max(0.0);
            let l = self.z.min(1.0).max(0.0);

            let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
            let h_prime = h / 60.0;
            let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
            match h_prime {
                v if 0.0 <= v && v <= 1.0 => V3::new(c, x, 0.0),
                v if 1.0 <= v && v <= 2.0 => V3::new(x, c, 0.0),
                v if 2.0 <= v && v <= 3.0 => V3::new(0.0, c, x),
                v if 3.0 <= v && v <= 4.0 => V3::new(0.0, x, c),
                v if 4.0 <= v && v <= 5.0 => V3::new(x, 0.0, c),
                v if 5.0 <= v && v <= 6.0 => V3::new(c, 0.0, x),
                _ => V3::fill(0.0),
            }
        }

        pub fn min(&self, other: Self) -> Self {
            Self::new(
                self.x.min(other.x),
                self.y.min(other.y),
                self.z.min(other.z),
            )
        }

        pub fn max(&self, other: Self) -> Self {
            Self::new(
                self.x.max(other.x),
                self.y.max(other.y),
                self.z.max(other.z),
            )
        }
    }

    impl<T: Add<Output = T> + Copy> Add<T> for V3<T> {
        type Output = Self;

        fn add(self, rhs: T) -> Self::Output {
            Self::new(self.x + rhs, self.y + rhs, self.z + rhs)
        }
    }

    impl<T: Add<Output = T> + Copy> AddAssign<T> for V3<T> {
        fn add_assign(&mut self, rhs: T) {
            self.x = self.x + rhs;
            self.y = self.y + rhs;
            self.z = self.z + rhs;
        }
    }

    impl<T: Sub<Output = T> + Copy> Sub<T> for V3<T> {
        type Output = Self;

        fn sub(self, rhs: T) -> Self::Output {
            Self::new(self.x - rhs, self.y - rhs, self.z - rhs)
        }
    }

    impl<T: Sub<Output = T> + Copy> SubAssign<T> for V3<T> {
        fn sub_assign(&mut self, rhs: T) {
            self.x = self.x - rhs;
            self.y = self.y - rhs;
            self.z = self.z - rhs;
        }
    }

    impl<T: Mul<Output = T> + Copy> Mul<T> for V3<T> {
        type Output = Self;

        fn mul(self, rhs: T) -> Self::Output {
            Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
        }
    }

    impl<T: Mul<Output = T> + Copy> MulAssign<T> for V3<T> {
        fn mul_assign(&mut self, rhs: T) {
            self.x = self.x * rhs;
            self.y = self.y * rhs;
            self.z = self.z * rhs;
        }
    }

    impl<T: Div<Output = T> + Copy> Div<T> for V3<T> {
        type Output = Self;

        fn div(self, rhs: T) -> Self::Output {
            Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
        }
    }

    impl<T: Div<Output = T> + Copy> DivAssign<T> for V3<T> {
        fn div_assign(&mut self, rhs: T) {
            self.x = self.x / rhs;
            self.y = self.y / rhs;
            self.z = self.z / rhs;
        }
    }

    impl<T: Add<Output = T> + Copy> Add for V3<T> {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
        }
    }

    impl<T: Add<Output = T> + Copy> AddAssign for V3<T> {
        fn add_assign(&mut self, rhs: Self) {
            self.x = self.x + rhs.x;
            self.y = self.y + rhs.y;
            self.z = self.z + rhs.z;
        }
    }

    impl<T: Sub<Output = T> + Copy> Sub for V3<T> {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
        }
    }

    impl<T: Sub<Output = T> + Copy> SubAssign for V3<T> {
        fn sub_assign(&mut self, rhs: Self) {
            self.x = self.x - rhs.x;
            self.y = self.y - rhs.y;
            self.z = self.z - rhs.z;
        }
    }

    impl<T: Mul<Output = T> + Copy> Mul for V3<T> {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
        }
    }

    impl<T: Mul<Output = T> + Copy> MulAssign for V3<T> {
        fn mul_assign(&mut self, rhs: Self) {
            self.x = self.x * rhs.x;
            self.y = self.y * rhs.y;
            self.z = self.z * rhs.z;
        }
    }

    impl<T: Div<Output = T> + Copy> Div for V3<T> {
        type Output = Self;

        fn div(self, rhs: Self) -> Self::Output {
            Self::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
        }
    }

    impl<T: Div<Output = T> + Copy> DivAssign for V3<T> {
        fn div_assign(&mut self, rhs: Self) {
            self.x = self.x / rhs.x;
            self.y = self.y / rhs.y;
            self.z = self.z / rhs.z;
        }
    }

    impl<T: Neg<Output = T>> Neg for V3<T> {
        type Output = Self;

        fn neg(self) -> Self::Output {
            V3::new(self.x.neg(), self.y.neg(), self.z.neg())
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub struct V4<T> {
        x: T,
        y: T,
        z: T,
        w: T,
    }

    impl<T> V4<T> {
        pub fn new(x: T, y: T, z: T, w: T) -> Self {
            Self { x, y, z, w }
        }
    }

    impl<T: Num> V4<T> {
        pub fn dot(&self, other: Self) -> T {
            self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
        }
    }

    impl<T: Copy> V4<T> {
        pub fn x(&self) -> T {
            self.x
        }

        pub fn y(&self) -> T {
            self.y
        }

        pub fn z(&self) -> T {
            self.z
        }

        pub fn w(&self) -> T {
            self.w
        }
    }

    impl<T: Add<Output = T> + Copy> Add for V4<T> {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self::new(
                self.x + rhs.x,
                self.y + rhs.y,
                self.z + rhs.z,
                self.w + rhs.w,
            )
        }
    }

    impl<T: Mul<Output = T> + Copy> Mul<T> for V4<T> {
        type Output = Self;

        fn mul(self, rhs: T) -> Self::Output {
            Self::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
        }
    }

    impl<T: Div<Output = T> + Copy> Div<T> for V4<T> {
        type Output = Self;

        fn div(self, rhs: T) -> Self::Output {
            Self::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs)
        }
    }

    #[derive(Debug, Copy)]
    pub struct M4<T> {
        pub c0: V4<T>,
        pub c1: V4<T>,
        pub c2: V4<T>,
        pub c3: V4<T>,
    }

    impl<T> M4<T> {
        pub fn new(c0: V4<T>, c1: V4<T>, c2: V4<T>, c3: V4<T>) -> M4<T> {
            M4 { c0, c1, c2, c3 }
        }

        pub fn transpose(self) -> M4<T> {
            M4 {
                c0: V4::new(self.c0.x, self.c1.x, self.c2.x, self.c3.x),
                c1: V4::new(self.c0.y, self.c1.y, self.c2.y, self.c3.y),
                c2: V4::new(self.c0.z, self.c1.z, self.c2.z, self.c3.z),
                c3: V4::new(self.c0.w, self.c1.w, self.c2.w, self.c3.w),
            }
        }
    }

    impl M4<f32> {
        pub fn identity() -> Self {
            Self::new(
                V4::new(1.0, 0.0, 0.0, 0.0),
                V4::new(0.0, 1.0, 0.0, 0.0),
                V4::new(0.0, 0.0, 1.0, 0.0),
                V4::new(0.0, 0.0, 0.0, 1.0),
            )
        }

        pub fn translation(translate: V3<f32>) -> Self {
            M4::new(
                V4::new(1.0, 0.0, 0.0, 0.0),
                V4::new(0.0, 1.0, 0.0, 0.0),
                V4::new(0.0, 0.0, 1.0, 0.0),
                V4::new(translate.x(), translate.y(), translate.z(), 1.0),
            )
        }

        pub fn rotation(rotate: V3<f32>) -> Self {
            let rotate = rotate * std::f32::consts::PI * 2.0;
            let (sin_x, cos_x) = rotate.x().sin_cos();
            let (sin_y, cos_y) = rotate.y().sin_cos();
            let (sin_z, cos_z) = rotate.z().sin_cos();

            let rotation_x = M4::new(
                V4::new(1.0, 0.0, 0.0, 0.0),
                V4::new(0.0, cos_x, sin_x, 0.0),
                V4::new(0.0, -sin_x, cos_x, 0.0),
                V4::new(0.0, 0.0, 0.0, 1.0),
            );
            let rotation_y = M4::new(
                V4::new(cos_y, 0.0, sin_y, 0.0),
                V4::new(0.0, 1.0, 0.0, 0.0),
                V4::new(-sin_y, 0.0, cos_y, 0.0),
                V4::new(0.0, 0.0, 0.0, 1.0),
            );
            let rotation_z = M4::new(
                V4::new(cos_z, -sin_z, 0.0, 0.0),
                V4::new(sin_z, cos_z, 0.0, 0.0),
                V4::new(0.0, 0.0, 1.0, 0.0),
                V4::new(0.0, 0.0, 0.0, 1.0),
            );

            rotation_x * rotation_y * rotation_z
        }

        pub fn scale(scale: V3<f32>) -> Self {
            M4::new(
                V4::new(scale.x(), 0.0, 0.0, 0.0),
                V4::new(0.0, scale.y(), 0.0, 0.0),
                V4::new(0.0, 0.0, scale.z(), 0.0),
                V4::new(0.0, 0.0, 0.0, 1.0),
            )
        }
    }

    impl<T: Clone> Clone for M4<T> {
        fn clone(&self) -> Self {
            Self::new(
                self.c0.clone(),
                self.c1.clone(),
                self.c2.clone(),
                self.c3.clone(),
            )
        }
    }

    impl<T: Num> Mul<M4<T>> for M4<T> {
        type Output = M4<T>;

        fn mul(self, rhs: M4<T>) -> Self::Output {
            let m = self.transpose();

            let c00 = m.c0.clone().dot(rhs.c0.clone());
            let c01 = m.c1.clone().dot(rhs.c0.clone());
            let c02 = m.c2.clone().dot(rhs.c0.clone());
            let c03 = m.c3.clone().dot(rhs.c0.clone());

            let c10 = m.c0.clone().dot(rhs.c1.clone());
            let c11 = m.c1.clone().dot(rhs.c1.clone());
            let c12 = m.c2.clone().dot(rhs.c1.clone());
            let c13 = m.c3.clone().dot(rhs.c1.clone());

            let c20 = m.c0.clone().dot(rhs.c2.clone());
            let c21 = m.c1.clone().dot(rhs.c2.clone());
            let c22 = m.c2.clone().dot(rhs.c2.clone());
            let c23 = m.c3.clone().dot(rhs.c2.clone());

            let c30 = m.c0.dot(rhs.c3.clone());
            let c31 = m.c1.dot(rhs.c3.clone());
            let c32 = m.c2.dot(rhs.c3.clone());
            let c33 = m.c3.dot(rhs.c3.clone());

            M4::new(
                V4::new(c00, c01, c02, c03),
                V4::new(c10, c11, c12, c13),
                V4::new(c20, c21, c22, c23),
                V4::new(c30, c31, c32, c33),
            )
        }
    }

    impl Mul<V3<f32>> for M4<f32> {
        type Output = V3<f32>;

        fn mul(self, rhs: V3<f32>) -> Self::Output {
            let m = self;
            let vx = m.c0 * rhs.x;
            let vy = m.c1 * rhs.y;
            let vz = m.c2 * rhs.z;
            let vw = m.c3 * 1.0;

            let v = vx + vy + vz + vw;
            let v = v / v.w;

            V3::new(v.x, v.y, v.z)
        }
    }

    impl<T: Num> Mul<V4<T>> for M4<T> {
        type Output = V4<T>;

        fn mul(self, rhs: V4<T>) -> Self::Output {
            let m = self;
            let vx = m.c0 * rhs.x;
            let vy = m.c1 * rhs.y;
            let vz = m.c2 * rhs.z;
            let vw = m.c3 * rhs.w;
            V4::new(
                vx.x + vy.x + vz.x + vw.x,
                vx.y + vy.y + vz.y + vw.y,
                vx.z + vy.z + vz.z + vw.z,
                vx.w + vy.w + vz.w + vw.w,
            )
        }
    }
}

#[cfg(feature = "simd")]
mod simd {
    use packed_simd::{f32x4, shuffle};
    use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

    use super::Num;

    #[derive(Copy, Clone, Debug)]
    pub struct V3 {
        inner: f32x4,
    }

    impl V3 {
        pub fn new(x: f32, y: f32, z: f32) -> Self {
            Self {
                inner: f32x4::new(x, y, z, 1.0),
            }
        }

        pub fn fill(v: f32) -> Self {
            Self {
                inner: f32x4::splat(v),
            }
        }

        pub fn zero() -> Self {
            Self {
                inner: f32x4::splat(0.0),
            }
        }

        pub fn x(&self) -> f32 {
            unsafe { self.inner.extract_unchecked(0) }
        }

        pub fn y(&self) -> f32 {
            unsafe { self.inner.extract_unchecked(1) }
        }

        pub fn z(&self) -> f32 {
            unsafe { self.inner.extract_unchecked(2) }
        }

        pub fn rand() -> Self {
            Self::new(f32::rand(), f32::rand(), f32::rand())
        }

        pub fn length_squared(&self) -> f32 {
            let x = self.inner * self.inner;
            let x = unsafe { x.replace_unchecked(3, 0.0) };
            x.sum()
        }

        pub fn length(&self) -> f32 {
            self.length_squared().sqrt()
        }

        pub fn unit(&self) -> Self {
            *self / self.length()
        }

        pub fn dot(&self, other: &Self) -> f32 {
            let x = self.inner * other.inner;
            let x = unsafe { x.replace_unchecked(3, 0.0) };
            x.sum()
        }

        pub fn cross(&self, other: &Self) -> Self {
            let x0: f32x4 = shuffle!(self.inner, [1, 2, 0, 3]);
            let x1: f32x4 = shuffle!(self.inner, [2, 0, 1, 3]);
            let y0: f32x4 = shuffle!(other.inner, [2, 0, 1, 3]);
            let y1: f32x4 = shuffle!(other.inner, [1, 2, 0, 3]);

            Self {
                inner: (x0 * y0) - (x1 * y1),
            }
        }

        pub fn random_in_unit_sphere() -> Self {
            loop {
                let x = f32::rand() * 2.0 - 1.0;
                let y = f32::rand() * 2.0 - 1.0;
                let z = f32::rand() * 2.0 - 1.0;

                let v = V3::new(x, y, z);
                if v.length_squared() >= 1.0 {
                    continue;
                }
                return v;
            }
        }

        pub fn random_in_unit_disk() -> Self {
            loop {
                let x = f32::rand() * 2.0 - 1.0;
                let y = f32::rand() * 2.0 - 1.0;

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
            self.x().abs() <= 3.0 * f32::EPSILON
                && self.y().abs() <= 3.0 * f32::EPSILON
                && self.z().abs() <= 3.0 * f32::EPSILON
        }

        pub fn reflect(&self, normal: Self) -> Self {
            *self - (normal * self.dot(&normal) * 2.0)
        }

        pub fn refract(&self, normal: Self, etai_over_etat: f32) -> Self {
            let cos_theta = self.neg().dot(&normal).min(1.0);
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
                v if 0.0 <= v && v <= 1.0 => V3::new(c, x, 0.0),
                v if 1.0 <= v && v <= 2.0 => V3::new(x, c, 0.0),
                v if 2.0 <= v && v <= 3.0 => V3::new(0.0, c, x),
                v if 3.0 <= v && v <= 4.0 => V3::new(0.0, x, c),
                v if 4.0 <= v && v <= 5.0 => V3::new(x, 0.0, c),
                v if 5.0 <= v && v <= 6.0 => V3::new(c, 0.0, x),
                _ => V3::fill(0.0),
            }
        }

        pub fn min(&self, other: Self) -> Self {
            Self {
                inner: self.inner.min(other.inner),
            }
        }

        pub fn max(&self, other: Self) -> Self {
            Self {
                inner: self.inner.max(other.inner),
            }
        }
    }

    impl Add<f32> for V3 {
        type Output = Self;

        fn add(self, rhs: f32) -> Self::Output {
            Self {
                inner: self.inner + rhs,
            }
        }
    }

    impl AddAssign<f32> for V3 {
        fn add_assign(&mut self, rhs: f32) {
            self.inner += rhs;
        }
    }

    impl Sub<f32> for V3 {
        type Output = Self;

        fn sub(self, rhs: f32) -> Self::Output {
            Self {
                inner: self.inner - rhs,
            }
        }
    }

    impl SubAssign<f32> for V3 {
        fn sub_assign(&mut self, rhs: f32) {
            self.inner -= rhs;
        }
    }

    impl Mul<f32> for V3 {
        type Output = Self;

        fn mul(self, rhs: f32) -> Self::Output {
            Self {
                inner: self.inner * rhs,
            }
        }
    }

    impl MulAssign<f32> for V3 {
        fn mul_assign(&mut self, rhs: f32) {
            self.inner *= rhs
        }
    }

    impl Div<f32> for V3 {
        type Output = Self;

        fn div(self, rhs: f32) -> Self::Output {
            Self {
                inner: self.inner / rhs,
            }
        }
    }

    impl DivAssign<f32> for V3 {
        fn div_assign(&mut self, rhs: f32) {
            self.inner /= rhs;
        }
    }

    impl Add for V3 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self {
                inner: self.inner + rhs.inner,
            }
        }
    }

    impl AddAssign for V3 {
        fn add_assign(&mut self, rhs: Self) {
            self.inner += rhs.inner
        }
    }

    impl Sub for V3 {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            Self {
                inner: self.inner - rhs.inner,
            }
        }
    }

    impl SubAssign for V3 {
        fn sub_assign(&mut self, rhs: Self) {
            self.inner -= rhs.inner
        }
    }

    impl Mul for V3 {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            Self {
                inner: self.inner * rhs.inner,
            }
        }
    }

    impl MulAssign for V3 {
        fn mul_assign(&mut self, rhs: Self) {
            self.inner *= rhs.inner
        }
    }

    impl Div for V3 {
        type Output = Self;

        fn div(self, rhs: Self) -> Self::Output {
            let rhs = unsafe { rhs.inner.replace_unchecked(3, 1.0) };
            Self {
                inner: self.inner / rhs,
            }
        }
    }

    impl DivAssign for V3 {
        fn div_assign(&mut self, rhs: Self) {
            let rhs = unsafe { rhs.inner.replace_unchecked(3, 1.0) };
            self.inner /= rhs
        }
    }

    impl Neg for V3 {
        type Output = Self;

        fn neg(self) -> Self::Output {
            Self {
                inner: self.inner.neg(),
            }
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub struct M4 {
        c0: f32x4,
        c1: f32x4,
        c2: f32x4,
        c3: f32x4,
    }

    impl M4 {
        pub fn new<T: Into<[f32; 4]>>(c0: T, c1: T, c2: T, c3: T) -> Self {
            let c0 = c0.into().into();
            let c1 = c1.into().into();
            let c2 = c2.into().into();
            let c3 = c3.into().into();

            M4 { c0, c1, c2, c3 }
        }

        pub fn transpose(self) -> Self {
            let a0: [f32; 4] = self.c0.into();
            let a1: [f32; 4] = self.c1.into();
            let a2: [f32; 4] = self.c2.into();
            let a3: [f32; 4] = self.c3.into();

            let c0 = f32x4::new(a0[0], a1[0], a2[0], a3[0]);
            let c1 = f32x4::new(a0[1], a1[1], a2[1], a3[1]);
            let c2 = f32x4::new(a0[2], a1[2], a2[2], a3[2]);
            let c3 = f32x4::new(a0[3], a1[3], a2[3], a3[3]);

            Self { c0, c1, c2, c3 }
        }

        pub fn identity() -> Self {
            let c0 = f32x4::new(1.0, 0.0, 0.0, 0.0);
            let c1 = f32x4::new(0.0, 1.0, 0.0, 0.0);
            let c2 = f32x4::new(0.0, 0.0, 1.0, 0.0);
            let c3 = f32x4::new(0.0, 0.0, 0.0, 1.0);
            Self { c0, c1, c2, c3 }
        }

        pub fn translation(translate: V3) -> Self {
            M4::new(
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [translate.x(), translate.y(), translate.z(), 1.0],
            )
        }

        pub fn rotation(rotate: V3) -> Self {
            let rotate = rotate * std::f32::consts::PI * 2.0;
            let (sin_x, cos_x) = rotate.x().sin_cos();
            let (sin_y, cos_y) = rotate.y().sin_cos();
            let (sin_z, cos_z) = rotate.z().sin_cos();

            let rotation_x = M4::new(
                [1.0, 0.0, 0.0, 0.0],
                [0.0, cos_x, sin_x, 0.0],
                [0.0, -sin_x, cos_x, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            );
            let rotation_y = M4::new(
                [cos_y, 0.0, sin_y, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-sin_y, 0.0, cos_y, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            );
            let rotation_z = M4::new(
                [cos_z, -sin_z, 0.0, 0.0],
                [sin_z, cos_z, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            );

            rotation_x * rotation_y * rotation_z
        }

        pub fn scale(scale: V3) -> Self {
            M4::new(
                [scale.x(), 0.0, 0.0, 0.0],
                [0.0, scale.y(), 0.0, 0.0],
                [0.0, 0.0, scale.z(), 0.0],
                [0.0, 0.0, 0.0, 1.0],
            )
        }
    }

    impl Mul for M4 {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            let m = self.transpose();

            let c00 = (m.c0 * rhs.c0).sum();
            let c01 = (m.c1 * rhs.c0).sum();
            let c02 = (m.c2 * rhs.c0).sum();
            let c03 = (m.c3 * rhs.c0).sum();

            let c10 = (m.c0 * rhs.c1).sum();
            let c11 = (m.c1 * rhs.c1).sum();
            let c12 = (m.c2 * rhs.c1).sum();
            let c13 = (m.c3 * rhs.c1).sum();

            let c20 = (m.c0 * rhs.c2).sum();
            let c21 = (m.c1 * rhs.c2).sum();
            let c22 = (m.c2 * rhs.c2).sum();
            let c23 = (m.c3 * rhs.c2).sum();

            let c30 = (m.c0 * rhs.c3).sum();
            let c31 = (m.c1 * rhs.c3).sum();
            let c32 = (m.c2 * rhs.c3).sum();
            let c33 = (m.c3 * rhs.c3).sum();

            Self {
                c0: f32x4::new(c00, c01, c02, c03),
                c1: f32x4::new(c10, c11, c12, c13),
                c2: f32x4::new(c20, c21, c22, c23),
                c3: f32x4::new(c30, c31, c32, c33),
            }
        }
    }

    impl Mul<V3> for M4 {
        type Output = V3;

        fn mul(self, rhs: V3) -> Self::Output {
            let vx = self.c0 * rhs.x();
            let vy = self.c1 * rhs.y();
            let vz = self.c2 * rhs.z();
            let vw = self.c3 * 1.0;

            let v = vx + vy + vz + vw;
            let w = unsafe { v.extract_unchecked(3) };
            let v = v / w;

            V3 { inner: v }
        }
    }
}
