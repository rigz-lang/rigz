use crate::number::Number;
use std::ops::Shl;

impl Shl for Number {
    type Output = Number;

    #[inline]
    fn shl(self, rhs: Self) -> Self::Output {
        let shift = match rhs {
            Number::Int(i) => i,
            Number::UInt(u) => u as i64,
            Number::Float(f) => f as i64,
        };

        match self {
            Number::Int(i) => Number::Int(i << shift),
            Number::UInt(u) => Number::UInt(u << shift),
            Number::Float(f) => Number::Float(f64::from_bits(f.to_bits() << shift)),
        }
    }
}
