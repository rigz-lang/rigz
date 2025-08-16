use crate::number::Number;
use std::ops::{BitOr, BitOrAssign};

impl BitOr for &Number {
    type Output = Number;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Int(i), rhs) => Number::Int(i | rhs.to_int()),
            (Number::Float(f), rhs) => {
                Number::Float(f64::from_bits(f.to_bits() | rhs.to_float().to_bits()))
            }
        }
    }
}

impl BitOrAssign<&Number> for Number {
    fn bitor_assign(&mut self, rhs: &Number) {
        match (self, rhs) {
            (Number::Int(i), rhs) => *i |= rhs.to_int(),
            (Number::Float(f), rhs) => {
                *f = f64::from_bits(f.to_bits() | rhs.to_float().to_bits())
            }
        }
    }
}
