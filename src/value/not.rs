use crate::value::Value;
use std::ops::Not;

impl Not for Value {
    type Output = Value;

    #[inline]
    fn not(self) -> Self::Output {
        match self {
            Value::None => Value::Bool(true),
            Value::Bool(b) => Value::Bool(!b),
            Value::Error(e) => Value::Error(e),
            v => Value::Bool(!v.to_bool()),
        }
    }
}
