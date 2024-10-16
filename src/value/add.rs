use std::ops::Add;
use crate::number::Number;
use crate::value::Value;
use crate::VMError;

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) => Value::Error(v),
            (_, Value::Error(v)) => Value::Error(v),
            (Value::None, rhs) => rhs,
            (lhs, Value::None) => lhs,
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(a | b),
            (Value::Bool(a), b) => Value::Bool(a | b.to_bool()),
            (b, Value::Bool(a)) => Value::Bool(a | b.to_bool()),
            (Value::Number(a), Value::Number(b)) => {
                match a + b {
                    Ok(n) => Value::Number(n),
                    Err(e) => Value::Error(e)
                }
            },
            (Value::String(a), Value::String(b)) => {
                let mut result = a.clone();
                result.push_str(b.as_str());
                Value::String(result)
            }
            (Value::List(a), Value::List(b)) => {
                let mut result = a.clone();
                result.extend(b);
                Value::List(result)
            }
            (Value::Map(a), Value::Map(b)) => {
                let mut result = a.clone();
                result.extend(b);
                Value::Map(result)
            }
            _ => todo!()
        }
    }
}