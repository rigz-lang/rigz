use std::ops::Neg;
use crate::value::Value;

impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        match self {
            Value::None => Value::None,
            Value::Bool(b) => Value::Bool(!b),
            Value::Number(n) => Value::Number(-n),
            v => v,
        }
    }
}