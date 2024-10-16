use crate::number::Number;
use crate::VMError;
use std::ops::Rem;

impl Rem for Number {
    type Output = Number;

    #[inline]
    fn rem(self, rhs: Self) -> Self::Output {
        Number(self.0 % rhs.0)
    }
}
