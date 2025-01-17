use crate::{ObjectValue, VMError};
use std::ops::BitOr;

impl BitOr for &ObjectValue {
    type Output = ObjectValue;
    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ObjectValue::Primitive(a), ObjectValue::Primitive(b)) => (a | b).into(),
            (ObjectValue::Tuple(a), ObjectValue::Tuple(b)) => {
                ObjectValue::Tuple(a.iter().zip(b).map(|(a, b)| a | b).collect())
            }
            (ObjectValue::Tuple(a), b) => ObjectValue::Tuple(a.iter().map(|a| a | b).collect()),
            (b, ObjectValue::Tuple(a)) => ObjectValue::Tuple(a.iter().map(|a| b | a).collect()),
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} | {rhs}")).into()
            }
        }
    }
}
