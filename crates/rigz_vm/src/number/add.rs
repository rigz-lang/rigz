use crate::number::Number;
use std::ops::Add;

impl Add for Number {
    type Output = Number;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Int(i), rhs) => Number::Int(i + rhs.to_int()),
            (Number::Float(f), rhs) => Number::Float(f + rhs.to_float()),
        }
    }
}
