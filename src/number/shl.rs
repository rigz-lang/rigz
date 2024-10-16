use crate::number::Number;
use std::ops::Shl;

impl Shl for Number {
    type Output = Number;

    #[inline]
    fn shl(self, rhs: Self) -> Self::Output {
        Number(((self.0 as i64) << rhs.to_int()) as f64)
    }
}
