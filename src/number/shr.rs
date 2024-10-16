use crate::number::Number;
use std::ops::Shr;

impl Shr for Number {
    type Output = Number;

    #[inline]
    fn shr(self, rhs: Self) -> Self::Output {
        Number(((self.0 as i64) >> rhs.to_int()) as f64)
    }
}
