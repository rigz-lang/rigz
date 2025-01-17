use crate::PrimitiveValue;
use std::ops::Neg;

impl Neg for &PrimitiveValue {
    type Output = PrimitiveValue;

    #[inline]
    fn neg(self) -> Self::Output {
        match self {
            PrimitiveValue::None => PrimitiveValue::None,
            PrimitiveValue::Bool(b) => PrimitiveValue::Bool(!b),
            PrimitiveValue::Number(n) => PrimitiveValue::Number(-n),
            PrimitiveValue::Range(n) => PrimitiveValue::Range(-n),
            v => v.clone(),
        }
    }
}
