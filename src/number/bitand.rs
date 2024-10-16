use crate::number::Number;
use std::ops::BitAnd;

impl BitAnd for Number {
    type Output = Number;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Number(f64::from_bits(self.0.to_bits() & rhs.0.to_bits()))
    }
}
