use crate::{ObjectValue, VMError};
use std::ops::{Deref, Sub};

impl Sub for &ObjectValue {
    type Output = ObjectValue;
    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (ObjectValue::Primitive(a), ObjectValue::Primitive(b)) => (a - b).into(),
            (ObjectValue::List(a), ObjectValue::List(b)) => {
                let mut result = a.clone();
                result.retain(|v| !b.contains(v));
                ObjectValue::List(result)
            }
            (ObjectValue::List(a), b) => {
                let mut result = a.clone();
                result.retain(|v| v.borrow().deref() != b);
                ObjectValue::List(result)
            }
            (ObjectValue::Map(a), ObjectValue::Map(b)) => {
                let mut result = a.clone();
                result.retain(|k, _| !b.contains_key(k));
                ObjectValue::Map(result)
            }
            (ObjectValue::Map(a), b) => {
                let mut result = a.clone();
                result.retain(|_, v| b != v.borrow().deref());
                ObjectValue::Map(result)
            }
            (ObjectValue::Tuple(a), ObjectValue::Tuple(b)) => {
                ObjectValue::Tuple(a.iter().zip(b).map(|(a, b)| a.borrow().deref() - b.borrow().deref()).map(|v| v.into()).collect())
            }
            (ObjectValue::Tuple(a), b) => ObjectValue::Tuple(a.iter().map(|a| a.borrow().deref() - b).map(|v| v.into()).collect()),
            (b, ObjectValue::Tuple(a)) => ObjectValue::Tuple(a.iter().map(|a| b - a.borrow().deref()).map(|v| v.into()).collect()),
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} - {rhs}")).into()
            }
        }
    }
}
