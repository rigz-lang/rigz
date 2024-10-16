use crate::number::Number;
use crate::Reverse;

impl Reverse for Number {
    type Output = Number;

    #[inline]
    fn reverse(&self) -> Self::Output {
        Number(f64::from_bits(self.0.to_bits().reverse_bits()))
    }
}
