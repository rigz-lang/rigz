use crate::value::Value;
use crate::Reverse;

impl Reverse for Value {
    type Output = Value;

    #[inline]
    fn reverse(&self) -> Self::Output {
        match self {
            Value::Number(n) => Value::Number(n.reverse()),
            Value::String(s) => {
                let s = s.chars().rev().collect();
                Value::String(s)
            }
            Value::List(l) => Value::List(l.iter().rev().cloned().collect()),
            Value::Tuple(l) => Value::Tuple(l.iter().rev().cloned().collect()),
            Value::Map(m) => {
                let mut r = m.clone();
                r.reverse();
                Value::Map(r)
            }
            v => v.clone(),
        }
    }
}
