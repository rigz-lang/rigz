use crate::number::Number;
use std::ops::Mul;

impl Mul for Number {
    type Output = Number;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Number(self.0 * rhs.0)
    }
}
