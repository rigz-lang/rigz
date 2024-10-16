use crate::number::Number;
use std::ops::Shl;

impl Shl for Number {
    type Output = Number;

    #[inline]
    fn shl(self, rhs: Self) -> Self::Output {
        Number(f64::from_bits(self.0.to_bits() << rhs.to_int()))
    }
}
