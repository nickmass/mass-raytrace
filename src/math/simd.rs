#![allow(dead_code)]

use core_simd::{f32x2, f32x4, simd_swizzle, SimdFloat};

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use super::F;

type Fx4 = f32x4;
type Fx2 = f32x2;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct V2 {
    inner: Fx2,
}
impl V2 {
    #[inline(always)]
    pub const fn new(x: F, y: F) -> Self {
        Self {
            inner: Fx2::from_array([x, y]),
        }
    }

    #[inline(always)]
    pub fn fill(v: F) -> Self {
        Self {
            inner: Fx2::splat(v),
        }
    }

    #[inline(always)]
    pub fn x(&self) -> F {
        self.inner.as_array()[0]
    }

    #[inline(always)]
    pub fn y(&self) -> F {
        self.inner.as_array()[1]
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct V3 {
    inner: Fx4,
}

impl V3 {
    #[inline(always)]
    pub const fn new(x: F, y: F, z: F) -> Self {
        Self {
            inner: Fx4::from_array([x, y, z, 1.0]),
        }
    }

    #[inline(always)]
    pub fn fill(v: F) -> Self {
        Self {
            inner: Fx4::splat(v),
        }
    }

    #[inline(always)]
    pub fn x(&self) -> F {
        self.inner.as_array()[0]
    }

    #[inline(always)]
    pub fn y(&self) -> F {
        self.inner.as_array()[1]
    }

    #[inline(always)]
    pub fn z(&self) -> F {
        self.inner.as_array()[2]
    }

    pub fn dot(&self, other: Self) -> F {
        let mut x = self.inner * other.inner;
        x.as_mut_array()[3] = 0.0;
        x.reduce_sum()
    }

    pub fn cross(&self, other: Self) -> Self {
        let x0: Fx4 = simd_swizzle!(self.inner, [1, 2, 0, 3]);
        let x1: Fx4 = simd_swizzle!(self.inner, [2, 0, 1, 3]);
        let y0: Fx4 = simd_swizzle!(other.inner, [2, 0, 1, 3]);
        let y1: Fx4 = simd_swizzle!(other.inner, [1, 2, 0, 3]);

        Self {
            inner: (x0 * y0) - (x1 * y1),
        }
    }

    pub fn min(&self, other: Self) -> Self {
        Self {
            inner: self.inner.simd_min(other.inner),
        }
    }

    pub fn max(&self, other: Self) -> Self {
        Self {
            inner: self.inner.simd_max(other.inner),
        }
    }

    pub fn powf(&self, pow: F) -> Self {
        let v = self.inner.to_array();

        let inner = Fx4::from_array([
            v[0].powf(pow),
            v[1].powf(pow),
            v[2].powf(pow),
            v[3].powf(pow),
        ]);

        Self { inner }
    }

    pub fn abs(&self) -> Self {
        Self {
            inner: self.inner.abs(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct V4 {
    inner: Fx4,
}

impl V4 {
    #[inline(always)]
    pub const fn new(x: F, y: F, z: F, w: F) -> Self {
        Self {
            inner: Fx4::from_array([x, y, z, w]),
        }
    }

    #[inline(always)]
    pub fn fill(v: F) -> Self {
        Self {
            inner: Fx4::splat(v),
        }
    }

    #[inline(always)]
    pub fn x(&self) -> F {
        self.inner.as_array()[0]
    }

    #[inline(always)]
    pub fn y(&self) -> F {
        self.inner.as_array()[1]
    }

    #[inline(always)]
    pub fn z(&self) -> F {
        self.inner.as_array()[2]
    }

    #[inline(always)]
    pub fn w(&self) -> F {
        self.inner.as_array()[3]
    }

    pub fn min(&self, other: Self) -> Self {
        Self {
            inner: self.inner.simd_min(other.inner),
        }
    }

    pub fn max(&self, other: Self) -> Self {
        Self {
            inner: self.inner.simd_max(other.inner),
        }
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
                    inner: self.inner.$func(Self::fill(other).inner),
                }
            }
        }

        impl $op<$name> for F {
            type Output = $name;

            #[inline(always)]
            fn $func(self, other: $name) -> Self::Output {
                $name {
                    inner: $name::fill(self).inner.$func(other.inner),
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
                self.inner = self.inner.$func(Self::fill(other).inner);
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
implement_vector!(operator, V3, Div, div, DivAssign, div_assign);

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
    pub const fn new(c0: V4, c1: V4, c2: V4, c3: V4) -> Self {
        M4 {
            c0: c0.inner,
            c1: c1.inner,
            c2: c2.inner,
            c3: c3.inner,
        }
    }

    pub fn transpose(self) -> Self {
        let a0: [F; 4] = self.c0.into();
        let a1: [F; 4] = self.c1.into();
        let a2: [F; 4] = self.c2.into();
        let a3: [F; 4] = self.c3.into();

        Self::new(
            [a0[0], a1[0], a2[0], a3[0]].into(),
            [a0[1], a1[1], a2[1], a3[1]].into(),
            [a0[2], a1[2], a2[2], a3[2]].into(),
            [a0[3], a1[3], a2[3], a3[3]].into(),
        )
    }

    fn transform(self, rhs: V3, w: F) -> V3 {
        let vx = self.c0 * Fx4::splat(rhs.x());
        let vy = self.c1 * Fx4::splat(rhs.y());
        let vz = self.c2 * Fx4::splat(rhs.z());
        let vw = self.c3 * Fx4::splat(w);

        let v = vx + vy + vz + vw;

        V3 { inner: v }
    }

    pub fn transform_vector(self, rhs: V3) -> V3 {
        self.transform(rhs, 0.0)
    }

    pub fn transform_point(self, rhs: V3) -> V3 {
        self.transform(rhs, 1.0)
    }
}

impl Mul for M4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let m = self.transpose();

        let c00 = (m.c0 * rhs.c0).reduce_sum();
        let c01 = (m.c1 * rhs.c0).reduce_sum();
        let c02 = (m.c2 * rhs.c0).reduce_sum();
        let c03 = (m.c3 * rhs.c0).reduce_sum();

        let c10 = (m.c0 * rhs.c1).reduce_sum();
        let c11 = (m.c1 * rhs.c1).reduce_sum();
        let c12 = (m.c2 * rhs.c1).reduce_sum();
        let c13 = (m.c3 * rhs.c1).reduce_sum();

        let c20 = (m.c0 * rhs.c2).reduce_sum();
        let c21 = (m.c1 * rhs.c2).reduce_sum();
        let c22 = (m.c2 * rhs.c2).reduce_sum();
        let c23 = (m.c3 * rhs.c2).reduce_sum();

        let c30 = (m.c0 * rhs.c3).reduce_sum();
        let c31 = (m.c1 * rhs.c3).reduce_sum();
        let c32 = (m.c2 * rhs.c3).reduce_sum();
        let c33 = (m.c3 * rhs.c3).reduce_sum();

        Self {
            c0: Fx4::from_array([c00, c01, c02, c03]),
            c1: Fx4::from_array([c10, c11, c12, c13]),
            c2: Fx4::from_array([c20, c21, c22, c23]),
            c3: Fx4::from_array([c30, c31, c32, c33]),
        }
    }
}
