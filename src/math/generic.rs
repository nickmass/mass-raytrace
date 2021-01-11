#![allow(dead_code)]

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use super::{Num, F};

impl V3<F> {
    pub fn dot(&self, other: Self) -> F {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
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

    pub fn powf(&self, pow: f32) -> Self {
        Self::new(self.x.powf(pow), self.y.powf(pow), self.z.powf(pow))
    }

    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }
}

impl<T: Mul<Output = T> + Add<Output = T> + Copy> V4<T> {
    fn dot(&self, other: Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }
}

impl V4<F> {
    pub fn min(&self, other: Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
            self.w.min(other.w),
        )
    }

    pub fn max(&self, other: Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
            self.w.max(other.w),
        )
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
    pub const fn new(c0: V4<T>, c1: V4<T>, c2: V4<T>, c3: V4<T>) -> Self {
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

impl M4<F> {
    fn transform(self, rhs: V3<F>, w: F) -> V3<F> {
        let vx = self.c0 * rhs.x;
        let vy = self.c1 * rhs.y;
        let vz = self.c2 * rhs.z;
        let vw = self.c3 * w;

        let v = vx + vy + vz + vw;

        V3::new(v.x, v.y, v.z)
    }

    pub fn transform_vector(self, rhs: V3<F>) -> V3<F> {
        self.transform(rhs, 0.0)
    }

    pub fn transform_point(self, rhs: V3<F>) -> V3<F> {
        self.transform(rhs, 1.0)
    }
}

impl Mul for M4<F> {
    type Output = M4<F>;

    fn mul(self, rhs: M4<F>) -> Self::Output {
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

impl Mul<V4<F>> for M4<F> {
    type Output = V4<F>;

    fn mul(self, rhs: V4<F>) -> Self::Output {
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

impl From<[F; 2]> for V2<F> {
    fn from(other: [F; 2]) -> Self {
        Self::new(other[0], other[1])
    }
}

impl From<[F; 3]> for V3<F> {
    fn from(other: [F; 3]) -> Self {
        Self::new(other[0], other[1], other[2])
    }
}

impl From<[F; 4]> for V4<F> {
    fn from(other: [F; 4]) -> Self {
        Self::new(other[0], other[1], other[2], other[3])
    }
}

macro_rules! implement_vector{
    (operator, $name:ident, $op:ident, $func:ident, $op_assign:ident, $func_assign:ident, $($field:ident),*) => {
        impl<T: $op<Output = T>> $op for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn $func(self, other: Self) -> Self::Output {
                $name {
                    $($field: self.$field.$func(other.$field),)*
                }
            }
        }

        impl<T: $op<Output = T> + Clone> $op<T> for $name<T> {
            type Output = Self;

            #[inline(always)]
            fn $func(self, other: T) -> Self::Output {
                $name {
                    $($field: self.$field.$func(other.clone()),)*
                }
            }
        }

        impl $op<$name<f32>> for f32 {
            type Output = $name<f32>;

            #[inline(always)]
            fn $func(self, other: $name<f32>) -> Self::Output {
                $name {
                    $($field: self.$func(other.$field),)*
                }
            }
        }

        impl $op<$name<f64>> for f64 {
            type Output = $name<f64>;

            #[inline(always)]
            fn $func(self, other: $name<f64>) -> Self::Output {
                $name {
                    $($field: self.$func(other.$field),)*
                }
            }
        }

        impl<T: $op<Output = T> + Clone> $op_assign for $name<T> {
            #[inline(always)]
            fn $func_assign(&mut self, other: Self) {
                *self = $name {
                    $($field: self.$field.clone().$func(other.$field),)*
                }
            }
        }

        impl<T: $op<Output = T> + Clone> $op_assign<T> for $name<T> {
            #[inline(always)]
            fn $func_assign(&mut self, other: T) {
                *self = $name {
                    $($field: self.$field.clone().$func(other.clone()),)*
                }
            }
        }

    };
    ($name:ident, $($field:ident),*) => {
        #[repr(C)]
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub struct $name<T> {
            $($field: T,)*
        }

        impl<T: Clone> $name<T> {
            pub fn fill(v: T) -> Self {
                $name {
                    $($field: v.clone(),)*
                }
            }

            $(
                #[inline(always)]
                pub fn $field(&self) -> T {
                    self.$field.clone()
                }
            )*
        }

        impl<T> $name<T> {
            pub const fn new($($field: T,)*) -> Self {
                $name {
                    $($field,)*
                }
            }
        }

        impl<T: Neg<Output = T>> Neg for $name<T> {
            type Output = Self;

            fn neg(self) -> Self::Output {
                $name {
                    $($field: self.$field.neg(),)*
                }
            }
        }


        implement_vector!(operator, $name, Add, add, AddAssign, add_assign, $($field),*);
        implement_vector!(operator, $name, Sub, sub, SubAssign, sub_assign, $($field),*);
        implement_vector!(operator, $name, Mul, mul, MulAssign, mul_assign, $($field),*);
        implement_vector!(operator, $name, Div, div, DivAssign, div_assign, $($field),*);
    }
}

implement_vector!(V2, x, y);
implement_vector!(V3, x, y, z);
implement_vector!(V4, x, y, z, w);
