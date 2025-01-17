use crate::ObjectValue;
use crate::ObjectValue::Primitive;
use crate::{PrimitiveValue, VMError};
use std::ops::Div;

impl Div for &ObjectValue {
    type Output = ObjectValue;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Primitive(PrimitiveValue::String(a)), Primitive(PrimitiveValue::String(b))) => {
                let result = a.split(b.as_str());
                ObjectValue::List(result.map(|s| s.into()).collect())
            }
            (Primitive(a), Primitive(b)) => (a / b).into(),
            (Primitive(PrimitiveValue::String(a)), b) => {
                let b = b.to_string();
                let result = a.split(b.as_str());
                ObjectValue::List(result.map(|s| s.into()).collect())
            }
            (ObjectValue::Tuple(a), ObjectValue::Tuple(b)) => {
                ObjectValue::Tuple(a.iter().zip(b).map(|(a, b)| a / b).collect())
            }
            (ObjectValue::Tuple(a), b) => ObjectValue::Tuple(a.iter().map(|a| a / b).collect()),
            (b, ObjectValue::Tuple(a)) => ObjectValue::Tuple(a.iter().map(|a| b / a).collect()),
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} / {rhs}")).into()
            }
        }
    }
}
