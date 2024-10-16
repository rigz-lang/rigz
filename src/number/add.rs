use crate::number::Number;
use std::ops::Add;

impl Add for Number {
    type Output = Number;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Number(self.0 + rhs.0)
    }
}
