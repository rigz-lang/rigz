use crate::number::Number;
use std::ops::{Div, DivAssign};

impl Div for &Number {
    type Output = Number;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Int(i), rhs) => Number::Int(i / rhs.to_int()),
            (Number::Float(f), rhs) => Number::Float(f / rhs.to_float()),
        }
    }
}

impl DivAssign<&Number> for Number {
    fn div_assign(&mut self, rhs: &Number) {
        match (self, rhs) {
            (Number::Int(i), rhs) => *i /= rhs.to_int(),
            (Number::Float(f), rhs) => *f /= rhs.to_float(),
        }
    }
}
