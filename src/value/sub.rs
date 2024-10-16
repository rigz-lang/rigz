use std::ops::{Sub};
use crate::value::Value;

impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) => Value::Error(v),
            (_, Value::Error(v)) => Value::Error(v),
            (Value::None, rhs) => -rhs,
            (lhs, Value::None) => lhs,
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(a | b),
            (Value::Bool(a), b) => Value::Bool(a | b.to_bool()),
            (b, Value::Bool(a)) => Value::Bool(a | b.to_bool()),
            (Value::Number(a), Value::Number(b)) => {
                match a - b {
                    Ok(n) => Value::Number(n),
                    Err(e) => Value::Error(e)
                }
            },
            (Value::String(a), Value::String(b)) => {
                let result = a.replacen(b.as_str(), "", 1);
                Value::String(result)
            }
            (Value::List(a), Value::List(b)) => {
                let mut result = a.clone();
                result.retain(|v| !b.contains(v));
                Value::List(result)
            }
            (Value::List(a), b) => {
                let mut result = a.clone();
                result.retain(|v| *v != b);
                Value::List(result)
            }
            (Value::Map(a), Value::Map(b)) => {
                let mut result = a.clone();
                result.retain(|k, _| !b.contains_key(k));
                Value::Map(result)
            }
            (Value::Map(a), b) => {
                let mut result = a.clone();
                result.retain(|_, v| b != *v);
                Value::Map(result)
            }
            _ => todo!()
        }
    }
}