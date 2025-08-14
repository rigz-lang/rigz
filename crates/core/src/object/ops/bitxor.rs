use crate::{ObjectValue, VMError};
use std::ops::{BitXor, Deref};

impl BitXor for &ObjectValue {
    type Output = ObjectValue;
    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ObjectValue::Primitive(a), ObjectValue::Primitive(b)) => (a ^ b).into(),
            (ObjectValue::Tuple(a), ObjectValue::Tuple(b)) => {
                ObjectValue::Tuple(a.iter().zip(b).map(|(a, b)| (a.borrow().deref() ^ b.borrow().deref()).into()).collect())
            }
            (ObjectValue::Tuple(a), b) => ObjectValue::Tuple(a.iter().map(|a| (a.borrow().deref() ^ b).into()).collect()),
            (b, ObjectValue::Tuple(a)) => ObjectValue::Tuple(a.iter().map(|a| (b ^ a.borrow().deref()).into()).collect()),
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} ^ {rhs}")).into()
            }
        }
    }
}
