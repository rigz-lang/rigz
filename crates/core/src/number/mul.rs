use crate::number::Number;
use std::ops::{Mul, MulAssign};

impl Mul for &Number {
    type Output = Number;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Int(i), rhs) => Number::Int(i * rhs.to_int()),
            (Number::Float(f), rhs) => Number::Float(f * rhs.to_float()),
        }
    }
}

impl MulAssign<&Number> for Number {
    fn mul_assign(&mut self, rhs: &Number) {
        match (self, rhs) {
            (Number::Int(i), rhs) => *i *= rhs.to_int(),
            (Number::Float(f), rhs) => *f *= rhs.to_float(),
        }
    }
}
