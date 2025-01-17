use crate::{AsPrimitive, ObjectValue};
use std::ops::Not;

impl Not for &ObjectValue {
    type Output = ObjectValue;

    fn not(self) -> Self::Output {
        match self {
            ObjectValue::Primitive(p) => p.not().into(),
            o => o.to_bool().not().into(),
        }
    }
}
