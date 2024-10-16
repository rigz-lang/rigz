use crate::number::Number;
use crate::Logical;

impl Logical<Number> for Number {
    type Output = Number;

    #[inline]
    fn and(self, rhs: Number) -> Self::Output {
        match (self.is_zero(), rhs.is_zero()) {
            (true, true) => rhs,
            (false, false) => self,
            (false, true) => self,
            (true, false) => rhs,
        }
    }

    #[inline]
    fn or(self, rhs: Number) -> Self::Output {
        match (self.is_zero(), rhs.is_zero()) {
            (true, true) => self,
            (false, false) => rhs,
            (false, true) => rhs,
            (true, false) => self,
        }
    }

    #[inline]
    fn xor(self, rhs: Number) -> Self::Output {
        match (self.is_zero(), rhs.is_zero()) {
            (true, true) => Number::zero(),
            (false, false) => Number::zero(),
            (false, true) => rhs,
            (true, false) => self,
        }
    }
}
