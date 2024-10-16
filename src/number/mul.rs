use crate::number::Number;
use std::ops::Mul;

impl Mul for Number {
    type Output = Number;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Int(i), rhs) => Number::Int(i * rhs.to_int()),
            (Number::Float(f), rhs) => Number::Float(f * rhs.to_float()),
        }
    }
}
