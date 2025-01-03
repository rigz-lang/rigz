use crate::number::Number;
use std::ops::BitOr;

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
