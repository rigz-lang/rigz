use crate::{ObjectValue, VMError};
use std::ops::{BitOr, BitOrAssign, Deref, DerefMut};

impl BitOr for &ObjectValue {
    type Output = ObjectValue;
    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ObjectValue::Primitive(a), ObjectValue::Primitive(b)) => (a | b).into(),
            (ObjectValue::Tuple(a), ObjectValue::Tuple(b)) => ObjectValue::Tuple(
                a.iter()
                    .zip(b)
                    .map(|(a, b)| (a.borrow().deref() | b.borrow().deref()).into())
                    .collect(),
            ),
            (ObjectValue::Tuple(a), b) => {
                ObjectValue::Tuple(a.iter().map(|a| (a.borrow().deref() | b).into()).collect())
            }
            (b, ObjectValue::Tuple(a)) => {
                ObjectValue::Tuple(a.iter().map(|a| (b | a.borrow().deref()).into()).collect())
            }
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} | {rhs}")).into()
            }
        }
    }
}

impl BitOrAssign<&ObjectValue> for ObjectValue {
    fn bitor_assign(&mut self, rhs: &ObjectValue) {
        match (self, rhs) {
            (ObjectValue::Primitive(a), ObjectValue::Primitive(b)) => *a |= b,
            (ObjectValue::Tuple(a), ObjectValue::Tuple(b)) => {
                for (a, b) in a.iter_mut().zip(b) {
                    *a.borrow_mut().deref_mut() |= b.borrow().deref();
                }
            }
            (ObjectValue::Tuple(a), b) => {
                for v in a {
                    *v.borrow_mut().deref_mut() |= b;
                }
            }
            (b, ObjectValue::Tuple(a)) => {
                *b = ObjectValue::Tuple(
                    a.iter()
                        .map(|a| (b.deref() | a.borrow().deref()).into())
                        .collect(),
                )
            }
            (lhs, rhs) => {
                *lhs = VMError::UnsupportedOperation(format!("Not supported: {lhs} | {rhs}")).into()
            }
        }
    }
}
