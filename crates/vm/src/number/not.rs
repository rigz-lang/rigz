use crate::number::Number;
use std::ops::Not;

impl Not for &Number {
    type Output = Number;

    #[inline]
    fn not(self) -> Self::Output {
        match self {
            Number::Int(v) => Number::Int(!v),
            Number::Float(v) => Number::Float(f64::from_bits(!v.to_bits())),
        }
    }
}
