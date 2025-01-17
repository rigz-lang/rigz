use crate::{IndexMap, Number, ObjectValue, PrimitiveValue, VMError};
use std::cell::RefCell;
use std::rc::Rc;

impl From<ObjectValue> for Rc<RefCell<ObjectValue>> {
    #[inline]
    fn from(value: ObjectValue) -> Self {
        Rc::new(RefCell::new(value))
    }
}

impl<T: Into<PrimitiveValue>> From<T> for ObjectValue {
    fn from(v: T) -> ObjectValue {
        ObjectValue::Primitive(v.into())
    }
}

impl<T: Into<ObjectValue>> From<Vec<T>> for ObjectValue {
    #[inline]
    fn from(value: Vec<T>) -> Self {
        ObjectValue::List(value.into_iter().map(|v| v.into()).collect())
    }
}

impl<K: Into<ObjectValue>, V: Into<ObjectValue>> From<IndexMap<K, V>> for ObjectValue {
    #[inline]
    fn from(value: IndexMap<K, V>) -> Self {
        ObjectValue::Map(
            value
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

impl<A: Into<ObjectValue>, B: Into<ObjectValue>> From<(A, B)> for ObjectValue {
    #[inline]
    fn from(value: (A, B)) -> Self {
        ObjectValue::Tuple(vec![value.0.into(), value.1.into()])
    }
}

impl<A: Into<ObjectValue>, B: Into<ObjectValue>, C: Into<ObjectValue>> From<(A, B, C)>
    for ObjectValue
{
    #[inline]
    fn from(value: (A, B, C)) -> Self {
        ObjectValue::Tuple(vec![value.0.into(), value.1.into(), value.2.into()])
    }
}

impl<A: Into<ObjectValue>, B: Into<ObjectValue>, C: Into<ObjectValue>, D: Into<ObjectValue>>
    From<(A, B, C, D)> for ObjectValue
{
    #[inline]
    fn from(value: (A, B, C, D)) -> Self {
        ObjectValue::Tuple(vec![
            value.0.into(),
            value.1.into(),
            value.2.into(),
            value.3.into(),
        ])
    }
}

impl<
        A: Into<ObjectValue>,
        B: Into<ObjectValue>,
        C: Into<ObjectValue>,
        D: Into<ObjectValue>,
        E: Into<ObjectValue>,
    > From<(A, B, C, D, E)> for ObjectValue
{
    #[inline]
    fn from(value: (A, B, C, D, E)) -> Self {
        ObjectValue::Tuple(vec![
            value.0.into(),
            value.1.into(),
            value.2.into(),
            value.3.into(),
            value.4.into(),
        ])
    }
}

impl<T: Into<ObjectValue>> From<Option<T>> for ObjectValue {
    fn from(value: Option<T>) -> Self {
        match value {
            None => ObjectValue::default(),
            Some(v) => v.into(),
        }
    }
}

impl<V: Into<ObjectValue>> From<Result<V, VMError>> for ObjectValue {
    #[inline]
    fn from(value: Result<V, VMError>) -> Self {
        match value {
            Ok(v) => v.into(),
            Err(e) => e.into(),
        }
    }
}
