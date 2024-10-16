use crate::number::Number;
use crate::Rev;

impl Rev for Number {
    type Output = Number;

    fn rev(self) -> Self::Output {
        match self {
            Number::Int(i) => Number::Int(i.reverse_bits()),
            Number::UInt(u) => Number::UInt(u.reverse_bits()),
            Number::Float(f) => Number::Float(f64::from_bits(f.to_bits().reverse_bits())),
        }
    }
}
