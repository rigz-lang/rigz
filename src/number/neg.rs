use crate::number::Number;
use std::ops::Neg;

impl Neg for Number {
    type Output = Number;

    #[inline]
    fn neg(self) -> Self::Output {
        match self {
            Number::Int(i) => Number::Int(-i),
            Number::UInt(i) => Number::UInt(!i + 1),
            Number::Float(f) => Number::Float(-f),
        }
    }
}
