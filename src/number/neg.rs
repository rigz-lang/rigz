use crate::number::Number;
use std::ops::Neg;

impl Neg for Number {
    type Output = Number;

    #[inline]
    fn neg(self) -> Self::Output {
        Number(-self.0)
    }
}
