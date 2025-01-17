use crate::ObjectValue;
use std::ops::Neg;

impl Neg for &ObjectValue {
    type Output = ObjectValue;

    fn neg(self) -> Self::Output {
        match self {
            ObjectValue::Primitive(p) => p.neg().into(),
            o => o.clone(),
        }
    }
}
