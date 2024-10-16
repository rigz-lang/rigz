use crate::number::Number;
use std::ops::Not;

impl Not for Number {
    type Output = Number;

    #[inline]
    fn not(self) -> Self::Output {
        Number(f64::from_bits(!self.0.to_bits()))
    }
}
