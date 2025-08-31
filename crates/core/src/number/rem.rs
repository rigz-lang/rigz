use crate::number::Number;
use std::ops::{Rem, RemAssign};

impl Rem for &Number {
    type Output = Number;

    #[inline]
    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Int(i), rhs) => Number::Int(i % rhs.to_int()),
            (Number::Float(f), rhs) => Number::Float(f % rhs.to_float()),
        }
    }
}

impl RemAssign<&Number> for Number {
    fn rem_assign(&mut self, rhs: &Number) {
        match (self, rhs) {
            (Number::Int(i), rhs) => *i %= rhs.to_int(),
            (Number::Float(f), rhs) => *f %= rhs.to_float(),
        }
    }
}
