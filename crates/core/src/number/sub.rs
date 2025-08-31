use crate::number::Number;
use std::ops::{Sub, SubAssign};

impl Sub for &Number {
    type Output = Number;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Int(i), rhs) => Number::Int(i - rhs.to_int()),
            (Number::Float(f), rhs) => Number::Float(f - rhs.to_float()),
        }
    }
}

impl SubAssign<&Number> for Number {
    fn sub_assign(&mut self, rhs: &Number) {
        match (self, rhs) {
            (Number::Int(i), rhs) => *i -= rhs.to_int(),
            (Number::Float(f), rhs) => *f -= rhs.to_float(),
        }
    }
}
