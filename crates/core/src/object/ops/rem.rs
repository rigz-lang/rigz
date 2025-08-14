use crate::ObjectValue;
use log::warn;
use std::ops::{Deref, Rem};

impl Rem for &ObjectValue {
    type Output = ObjectValue;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ObjectValue::Primitive(a), ObjectValue::Primitive(b)) => a.rem(b).into(),
            (ObjectValue::Tuple(a), ObjectValue::Tuple(b)) => ObjectValue::Tuple(
                a.iter()
                    .zip(b)
                    .map(|(a, b)| a.borrow().deref() % b.borrow().deref())
                    .map(|v| v.into())
                    .collect(),
            ),
            (ObjectValue::Tuple(a), b) => ObjectValue::Tuple(
                a.iter()
                    .map(|a| a.borrow().deref() % b)
                    .map(|v| v.into())
                    .collect(),
            ),
            (b, ObjectValue::Tuple(a)) => ObjectValue::Tuple(
                a.iter()
                    .map(|a| b % a.borrow().deref())
                    .map(|v| v.into())
                    .collect(),
            ),
            (a, b) => {
                warn!("{a} % {b} not implemented, defaulting to a - b");
                a - b
            }
        }
    }
}
