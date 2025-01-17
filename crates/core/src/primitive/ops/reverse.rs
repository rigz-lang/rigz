use crate::{PrimitiveValue, Reverse};

impl Reverse for PrimitiveValue {
    type Output = PrimitiveValue;

    #[inline]
    fn reverse(&self) -> Self::Output {
        match self {
            PrimitiveValue::Number(n) => PrimitiveValue::Number(n.reverse()),
            PrimitiveValue::String(s) => {
                let s = s.chars().rev().collect();
                PrimitiveValue::String(s)
            }
            v => v.clone(),
        }
    }
}
