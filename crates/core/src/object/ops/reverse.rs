use crate::{ObjectValue, Reverse};

impl Reverse for ObjectValue {
    type Output = ObjectValue;

    fn reverse(&self) -> Self::Output {
        match self {
            ObjectValue::Primitive(p) => p.reverse().into(),
            ObjectValue::List(l) => ObjectValue::List(l.iter().rev().cloned().collect()),
            ObjectValue::Tuple(l) => ObjectValue::Tuple(l.iter().rev().cloned().collect()),
            ObjectValue::Map(m) => {
                let mut r = m.clone();
                r.reverse();
                ObjectValue::Map(r)
            }
            ObjectValue::Object(o) => o.reverse().unwrap_or_else(|e| e.into()),
        }
    }
}
