use std::ops::Neg;
use crate::number::Number;

impl Neg for Number {
    type Output = Number;

    fn neg(self) -> Self::Output {
        match self {
            Number::Int(i) => Number::Int(-i),
            Number::UInt(i) => Number::UInt(!i + 1),
            Number::Float(f) => Number::Float(-f),
        }
    }
}