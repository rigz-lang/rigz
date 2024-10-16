use crate::Rev;
use crate::value::Value;

impl Rev for Value {
    type Output = Value;

    fn rev(self) -> Self::Output {
        match self {
            Value::Number(n) => Value::Number(n.rev()),
            Value::String(s) => {
                let s = s.chars().rev().collect();
                Value::String(s)
            },
            Value::List(l) => {
                Value::List(l.iter().rev().cloned().collect())
            }
            Value::Map(m) => {
                let mut r = m.clone();
                r.reverse();
                Value::Map(r)
            }
            v => v,
        }
    }
}