use crate::number::Number;
use crate::VMError;
use std::ops::BitOr;

impl BitOr for Number {
    type Output = Number;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Number(f64::from_bits(self.0.to_bits() | rhs.0.to_bits()))
    }
}
