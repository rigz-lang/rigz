use std::ops::Neg;
use crate::value::Value;

impl <'vm> Neg for Value<'vm> {
    type Output = Value<'vm>;

    fn neg(self) -> Self::Output {
        match self {
            Value::None => Value::None,
            Value::Bool(b) => Value::Bool(!b),
            Value::Number(n) => Value::Number(-n),
            v => v,
        }
    }
}