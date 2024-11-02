use crate::value::Value;
use std::ops::Neg;

impl Neg for Value {
    type Output = Value;

    #[inline]
    fn neg(self) -> Self::Output {
        match self {
            Value::None => Value::None,
            Value::Bool(b) => Value::Bool(!b),
            Value::Number(n) => Value::Number(-n),
            Value::Range(n) => Value::Range(-n),
            v => v,
        }
    }
}
