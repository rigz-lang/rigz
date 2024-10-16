use std::ops::Shr;
use crate::number::Number;

impl Shr for Number {
    type Output = Number;

    fn shr(self, rhs: Self) -> Self::Output {
        let shift = match rhs {
            Number::Int(i) => i,
            Number::UInt(u) => u as i64,
            Number::Float(f) => f as i64,
        };

        match self {
            Number::Int(i) => Number::Int(i >> shift),
            Number::UInt(u) => Number::UInt(u >> shift),
            Number::Float(f) => Number::Float(f64::from_bits(f.to_bits() >> shift))
        }
    }
}