use indexmap::IndexMap;
use crate::Rev;
use crate::value::Value;

impl Rev for Value {
    type Output = Value;

    fn rev(self) -> Self::Output {
        match self {
            Value::Number(n) => Value::Number(n.rev()),
            Value::String(s) => {
                let mut bytes = s.as_bytes();
                bytes.reverse();
                Value::String(String::from(bytes))
            },
            Value::List(l) => {
                Value::List(l.iter().rev().collect())
            }
            Value::Map(m) => {
                Value::Map(IndexMap::from(m.iter().rev().collect()))
            }
            v => v,
        }
    }
}