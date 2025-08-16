use crate::{ObjectValue, VMError};
use std::ops::{Deref, DerefMut, Sub, SubAssign};

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
            (ObjectValue::Tuple(a), ObjectValue::Tuple(b)) => ObjectValue::Tuple(
                a.iter()
                    .zip(b)
                    .map(|(a, b)| a.borrow().deref() - b.borrow().deref())
                    .map(|v| v.into())
                    .collect(),
            ),
            (ObjectValue::Tuple(a), b) => ObjectValue::Tuple(
                a.iter()
                    .map(|a| a.borrow().deref() - b)
                    .map(|v| v.into())
                    .collect(),
            ),
            (b, ObjectValue::Tuple(a)) => ObjectValue::Tuple(
                a.iter()
                    .map(|a| b - a.borrow().deref())
                    .map(|v| v.into())
                    .collect(),
            ),
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} - {rhs}")).into()
            }
        }
    }
}

impl SubAssign<&ObjectValue> for ObjectValue {
    fn sub_assign(&mut self, rhs: &ObjectValue) {
        match (self, rhs) {
            (ObjectValue::Primitive(a), ObjectValue::Primitive(b)) => *a -= b,
            (ObjectValue::List(a), ObjectValue::List(b)) => {
                a.retain(|v| !b.contains(v));
            }
            (ObjectValue::List(a), b) => {
                a.retain(|v| v.borrow().deref() != b);
            }
            (ObjectValue::Map(a), ObjectValue::Map(b)) => {
                a.retain(|k, _| !b.contains_key(k));
            }
            (ObjectValue::Map(a), b) => {
                a.retain(|_, v| b != v.borrow().deref());
            }
            (ObjectValue::Tuple(a), ObjectValue::Tuple(b)) => {
                for (a, b) in a.iter_mut().zip(b) {
                    *a.borrow_mut().deref_mut() -= b.borrow().deref();
                }
            },
            (ObjectValue::Tuple(a), b) => {
                for v in a {
                    *v.borrow_mut().deref_mut() -= b;
                }
            },
            (b, ObjectValue::Tuple(a)) =>
                *b = ObjectValue::Tuple(a.iter().map(|a| (b.deref() + a.borrow().deref()).into()).collect()),
            (lhs, rhs) => {
                *lhs = VMError::UnsupportedOperation(format!("Not supported: {lhs} - {rhs}")).into()
            }
        }
    }
}