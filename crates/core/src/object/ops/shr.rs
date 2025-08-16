use crate::{ObjectValue, VMError};
use std::ops::{Deref, Shr, ShrAssign};

impl Shr for &ObjectValue {
    type Output = ObjectValue;

    fn shr(self, other: Self) -> Self::Output {
        match (self, other) {
            (ObjectValue::Primitive(lhs), ObjectValue::Primitive(rhs)) => lhs.shr(rhs).into(),
            (ObjectValue::Tuple(a), ObjectValue::Tuple(b)) => ObjectValue::Tuple(
                a.iter()
                    .zip(b)
                    .map(|(a, b)| a.borrow().deref() >> b.borrow().deref())
                    .map(|v| v.into())
                    .collect(),
            ),
            (ObjectValue::Tuple(a), b) => ObjectValue::Tuple(
                a.iter()
                    .map(|a| a.borrow().deref() >> b)
                    .map(|v| v.into())
                    .collect(),
            ),
            (b, ObjectValue::Tuple(a)) => ObjectValue::Tuple(
                a.iter()
                    .map(|a| b >> a.borrow().deref())
                    .map(|v| v.into())
                    .collect(),
            ),
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} >> {rhs}")).into()
            }
        }
    }
}

impl ShrAssign<&ObjectValue> for ObjectValue {
    fn shr_assign(&mut self, rhs: &ObjectValue) {
        *self = self.shr(rhs)
    }
}
