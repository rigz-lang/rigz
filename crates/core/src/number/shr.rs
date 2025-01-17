use crate::number::Number;
use std::ops::Shr;

impl Shr for &Number {
    type Output = Number;

    #[inline]
    fn shr(self, rhs: Self) -> Self::Output {
        let shift = match rhs {
            Number::Int(i) => i,
            Number::Float(f) => &(*f as i64),
        };

        match self {
            Number::Int(i) => Number::Int(i >> shift),
            Number::Float(f) => Number::Float(f64::from_bits(f.to_bits() >> shift)),
        }
    }
}
