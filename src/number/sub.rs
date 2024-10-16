use crate::number::Number;
use std::ops::Sub;

impl Sub for Number {
    type Output = Number;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Number(self.0 - rhs.0)
    }
}
