#![allow(dead_code)]

use packed_simd::{f32x2, f32x4, shuffle};

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use super::F;

type Fx4 = f32x4;
type Fx2 = f32x2;

#[derive(Copy, Clone, Debug)]
pub struct V2 {
    inner: Fx2,
}
impl V2 {
    pub fn new(x: F, y: F) -> Self {
        Self {
            inner: Fx2::new(x, y),
        }
    }

    pub fn fill(v: F) -> Self {
        Self {
            inner: Fx2::splat(v),
        }
    }

    pub fn zero() -> Self {
        Self {
            inner: Fx2::splat(0.0),
        }
    }

    #[inline(always)]
    pub fn x(&self) -> F {
        unsafe { self.inner.extract_unchecked(0) }
    }

    #[inline(always)]
    pub fn y(&self) -> F {
        unsafe { self.inner.extract_unchecked(1) }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct V3 {
    inner: Fx4,
}

impl V3 {
    pub fn new(x: F, y: F, z: F) -> Self {
        Self {
            inner: Fx4::new(x, y, z, 1.0),
        }
    }

    pub fn fill(v: F) -> Self {
        Self {
            inner: Fx4::splat(v),
        }
    }

    pub fn zero() -> Self {
        Self {
            inner: Fx4::splat(0.0),
        }
    }

    #[inline(always)]
    pub fn x(&self) -> F {
        unsafe { self.inner.extract_unchecked(0) }
    }

    #[inline(always)]
    pub fn y(&self) -> F {
        unsafe { self.inner.extract_unchecked(1) }
    }

    #[inline(always)]
    pub fn z(&self) -> F {
        unsafe { self.inner.extract_unchecked(2) }
    }

    pub fn dot(&self, other: Self) -> F {
        let x = self.inner * other.inner;
        let x = unsafe { x.replace_unchecked(3, 0.0) };
        x.sum()
    }

    pub fn cross(&self, other: Self) -> Self {
        let x0: Fx4 = shuffle!(self.inner, [1, 2, 0, 3]);
        let x1: Fx4 = shuffle!(self.inner, [2, 0, 1, 3]);
        let y0: Fx4 = shuffle!(other.inner, [2, 0, 1, 3]);
        let y1: Fx4 = shuffle!(other.inner, [1, 2, 0, 3]);

        Self {
            inner: (x0 * y0) - (x1 * y1),
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

    pub fn powf(&self, pow: F) -> Self {
        Self {
            inner: self.inner.powf(Self::fill(pow).inner),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct V4 {
    inner: Fx4,
}

impl V4 {
    pub fn new(x: F, y: F, z: F, w: F) -> Self {
        Self {
            inner: Fx4::new(x, y, z, w),
        }
    }

    pub fn fill(v: F) -> Self {
        Self {
            inner: Fx4::splat(v),
        }
    }

    pub fn zero() -> Self {
        Self {
            inner: Fx4::splat(0.0),
        }
    }

    #[inline(always)]
    pub fn x(&self) -> F {
        unsafe { self.inner.extract_unchecked(0) }
    }

    #[inline(always)]
    pub fn y(&self) -> F {
        unsafe { self.inner.extract_unchecked(1) }
    }

    #[inline(always)]
    pub fn z(&self) -> F {
        unsafe { self.inner.extract_unchecked(2) }
    }

    #[inline(always)]
    pub fn w(&self) -> F {
        unsafe { self.inner.extract_unchecked(3) }
    }
}

macro_rules! implement_vector {
    (operator, $name:ident, $op:ident, $func:ident, $op_assign:ident, $func_assign:ident $(, ($replace:literal, $value:literal))*) => {
        impl $op for $name {
            type Output = Self;

            #[inline(always)]
            fn $func(self, other: Self) -> Self::Output {
                Self {
                    inner: self.inner.$func(other.inner),
                }
            }
        }

        impl $op<F> for $name {
            type Output = Self;

            #[inline(always)]
            fn $func(self, other: F) -> Self::Output {
                Self {
                    inner: self.inner.$func(other),
                }
            }
        }

        impl $op_assign for $name {
            #[inline(always)]
            fn $func_assign(&mut self, other: Self) {
                self.inner = self.inner.$func(other.inner);
            }
        }

        impl $op_assign<F> for $name {
            #[inline(always)]
            fn $func_assign(&mut self, other: F) {
                self.inner = self.inner.$func(other);
            }
        }
    };
}

implement_vector!(operator, V2, Add, add, AddAssign, add_assign);
implement_vector!(operator, V2, Sub, sub, SubAssign, sub_assign);
implement_vector!(operator, V2, Mul, mul, MulAssign, mul_assign);
implement_vector!(operator, V2, Div, div, DivAssign, div_assign);

implement_vector!(operator, V3, Add, add, AddAssign, add_assign);
implement_vector!(operator, V3, Sub, sub, SubAssign, sub_assign);
implement_vector!(operator, V3, Mul, mul, MulAssign, mul_assign);

implement_vector!(operator, V4, Add, add, AddAssign, add_assign);
implement_vector!(operator, V4, Sub, sub, SubAssign, sub_assign);
implement_vector!(operator, V4, Mul, mul, MulAssign, mul_assign);
implement_vector!(operator, V4, Div, div, DivAssign, div_assign);

impl From<[F; 2]> for V2 {
    fn from(other: [F; 2]) -> Self {
        Self::new(other[0], other[1])
    }
}

impl Neg for V2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            inner: self.inner.neg(),
        }
    }
}

impl From<[F; 3]> for V3 {
    fn from(other: [F; 3]) -> Self {
        Self::new(other[0], other[1], other[2])
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

impl Div for V3 {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        let other = unsafe { other.inner.replace_unchecked(3, 1.0) };
        Self {
            inner: self.inner.div(other),
        }
    }
}

impl Div<F> for V3 {
    type Output = Self;
    fn div(self, other: F) -> Self {
        Self {
            inner: self.inner.div(other),
        }
    }
}

impl DivAssign for V3 {
    fn div_assign(&mut self, other: Self) {
        let other = unsafe { other.inner.replace_unchecked(3, 1.0) };
        self.inner = self.inner.div(other);
    }
}

impl DivAssign<F> for V3 {
    fn div_assign(&mut self, other: F) {
        self.inner = self.inner.div(other);
    }
}

impl From<[F; 4]> for V4 {
    fn from(other: [F; 4]) -> Self {
        Self::new(other[0], other[1], other[2], other[3])
    }
}

impl Neg for V4 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            inner: self.inner.neg(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct M4 {
    c0: Fx4,
    c1: Fx4,
    c2: Fx4,
    c3: Fx4,
}

impl M4 {
    pub fn new<T: Into<V4>>(c0: T, c1: T, c2: T, c3: T) -> Self {
        let c0 = c0.into().inner;
        let c1 = c1.into().inner;
        let c2 = c2.into().inner;
        let c3 = c3.into().inner;

        M4 { c0, c1, c2, c3 }
    }

    pub fn transpose(self) -> Self {
        let a0: [F; 4] = self.c0.into();
        let a1: [F; 4] = self.c1.into();
        let a2: [F; 4] = self.c2.into();
        let a3: [F; 4] = self.c3.into();

        Self::new(
            [a0[0], a1[0], a2[0], a3[0]],
            [a0[1], a1[1], a2[1], a3[1]],
            [a0[2], a1[2], a2[2], a3[2]],
            [a0[3], a1[3], a2[3], a3[3]],
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
            c0: Fx4::new(c00, c01, c02, c03),
            c1: Fx4::new(c10, c11, c12, c13),
            c2: Fx4::new(c20, c21, c22, c23),
            c3: Fx4::new(c30, c31, c32, c33),
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
