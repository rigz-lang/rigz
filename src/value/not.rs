use crate::value::Value;
use std::ops::Not;

impl<'vm> Not for Value<'vm> {
    type Output = Value<'vm>;

    #[inline]
    fn not(self) -> Self::Output {
        match self {
            Value::None => Value::Bool(true),
            Value::Bool(b) => Value::Bool(!b),
            Value::Number(n) => Value::Number(!n),
            Value::Error(e) => Value::Error(e),
            v => Value::Bool(!v.to_bool()),
        }
    }
}
