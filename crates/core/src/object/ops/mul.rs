use crate::ObjectValue;
use crate::ObjectValue::Primitive;
use crate::{PrimitiveValue, VMError};
use std::cell::RefCell;
use std::ops::{Deref, Mul, MulAssign};
use std::rc::Rc;

impl Mul for &ObjectValue {
    type Output = ObjectValue;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Primitive(PrimitiveValue::String(a)), Primitive(PrimitiveValue::String(b))) => {
                &ObjectValue::List(vec![Rc::new(RefCell::new(a.clone().into()))])
                    * &b.clone().into()
            }
            (Primitive(a), Primitive(b)) => (a * b).into(),
            (ObjectValue::Tuple(a), ObjectValue::Tuple(b)) => ObjectValue::Tuple(
                a.iter()
                    .zip(b)
                    .map(|(a, b)| a.borrow().deref() * b.borrow().deref())
                    .map(|v| v.into())
                    .collect(),
            ),
            (ObjectValue::Tuple(a), b) => ObjectValue::Tuple(
                a.iter()
                    .map(|a| a.borrow().deref() * b)
                    .map(|v| v.into())
                    .collect(),
            ),
            (b, ObjectValue::Tuple(a)) => ObjectValue::Tuple(
                a.iter()
                    .map(|a| b * a.borrow().deref())
                    .map(|v| v.into())
                    .collect(),
            ),
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} * {rhs}")).into()
            }
        }
    }
}

impl MulAssign<&ObjectValue> for ObjectValue {
    fn mul_assign(&mut self, rhs: &ObjectValue) {
        *self = self.mul(rhs);
    }
}
