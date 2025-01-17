use crate::{ObjectValue, VMError};
use std::ops::Add;

impl Add for &ObjectValue {
    type Output = ObjectValue;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ObjectValue::Primitive(a), ObjectValue::Primitive(b)) => (a + b).into(),
            (ObjectValue::Tuple(a), ObjectValue::Tuple(b)) => {
                ObjectValue::Tuple(a.iter().zip(b).map(|(a, b)| a + b).collect())
            }
            (ObjectValue::Tuple(a), b) => ObjectValue::Tuple(a.iter().map(|a| a + b).collect()),
            (b, ObjectValue::Tuple(a)) => ObjectValue::Tuple(a.iter().map(|a| b + a).collect()),
            (ObjectValue::List(a), ObjectValue::List(b)) => {
                let mut result = a.clone();
                result.extend(b.clone());
                ObjectValue::List(result)
            }
            (ObjectValue::List(a), b) => {
                let mut result = a.clone();
                result.push(b.clone());
                ObjectValue::List(result)
            }
            (b, ObjectValue::List(a)) => {
                let mut result = Vec::with_capacity(a.len() + 1);
                result.push(b.clone());
                result.extend(a.clone());
                ObjectValue::List(result)
            }
            (ObjectValue::Map(a), ObjectValue::Map(b)) => {
                let mut result = a.clone();
                result.extend(b.clone());
                ObjectValue::Map(result)
            }
            (ObjectValue::Map(a), b) | (b, ObjectValue::Map(a)) => {
                let mut result = a.clone();
                result.insert(b.clone(), b.clone());
                ObjectValue::Map(result)
            }
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} + {rhs}")).into()
            }
        }
    }
}
