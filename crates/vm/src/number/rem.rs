use crate::number::Number;
use std::ops::Rem;

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
