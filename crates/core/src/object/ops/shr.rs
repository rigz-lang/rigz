use crate::{ObjectValue, VMError};
use std::ops::Shr;

impl Shr for &ObjectValue {
    type Output = ObjectValue;

    fn shr(self, other: Self) -> Self::Output {
        match (self, other) {
            (ObjectValue::Primitive(lhs), ObjectValue::Primitive(rhs)) => lhs.shr(rhs).into(),
            (ObjectValue::Tuple(a), ObjectValue::Tuple(b)) => {
                ObjectValue::Tuple(a.iter().zip(b).map(|(a, b)| a >> b).collect())
            }
            (ObjectValue::Tuple(a), b) => ObjectValue::Tuple(a.iter().map(|a| a >> b).collect()),
            (b, ObjectValue::Tuple(a)) => ObjectValue::Tuple(a.iter().map(|a| b >> a).collect()),
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} >> {rhs}")).into()
            }
        }
    }
}
