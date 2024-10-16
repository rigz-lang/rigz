use crate::number::Number;
use std::ops::Div;

impl Div for Number {
    type Output = Number;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Number(self.0 / rhs.0)
    }
}
