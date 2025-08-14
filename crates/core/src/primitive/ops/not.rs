use crate::{PrimitiveValue, ToBool};
use std::ops::Not;

impl Not for &PrimitiveValue {
    type Output = PrimitiveValue;

    #[inline]
    fn not(self) -> Self::Output {
        match self {
            PrimitiveValue::None => PrimitiveValue::Bool(true),
            PrimitiveValue::Bool(b) => PrimitiveValue::Bool(!b),
            PrimitiveValue::Error(e) => PrimitiveValue::Error(e.clone()),
            v => PrimitiveValue::Bool(v.to_bool().not()),
        }
    }
}
