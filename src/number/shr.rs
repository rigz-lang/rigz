use crate::number::Number;
use std::ops::Shr;

impl Shr for Number {
    type Output = Number;

    #[inline]
    fn shr(self, rhs: Self) -> Self::Output {
        Number(f64::from_bits(self.0.to_bits() >> rhs.to_int()))
    }
}
