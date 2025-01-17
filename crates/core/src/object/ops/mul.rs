use crate::ObjectValue;
use crate::ObjectValue::Primitive;
use crate::{PrimitiveValue, VMError};
use std::ops::Mul;

impl Mul for &ObjectValue {
    type Output = ObjectValue;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Primitive(PrimitiveValue::String(a)), Primitive(PrimitiveValue::String(b))) => {
                &ObjectValue::List(vec![a.clone().into()]) * &b.clone().into()
            }
            (Primitive(a), Primitive(b)) => (a * b).into(),
            (ObjectValue::Tuple(a), ObjectValue::Tuple(b)) => {
                ObjectValue::Tuple(a.iter().zip(b).map(|(a, b)| a * b).collect())
            }
            (ObjectValue::Tuple(a), b) => ObjectValue::Tuple(a.iter().map(|a| a * b).collect()),
            (b, ObjectValue::Tuple(a)) => ObjectValue::Tuple(a.iter().map(|a| b * a).collect()),
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} * {rhs}")).into()
            }
        }
    }
}
