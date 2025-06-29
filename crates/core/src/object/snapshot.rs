use crate::{ObjectValue, Snapshot, VMError};
use std::fmt::Display;
use std::vec::IntoIter;

impl Snapshot for ObjectValue {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            ObjectValue::Primitive(p) => {
                let mut res = vec![1];
                res.extend(p.as_bytes());
                res
            }
            ObjectValue::List(v) => {
                let mut res = vec![2];
                res.extend(v.as_bytes());
                res
            }
            ObjectValue::Map(m) => {
                let mut res = vec![3];
                res.extend(m.as_bytes());
                res
            }
            ObjectValue::Tuple(v) => {
                let mut res = vec![4];
                res.extend(v.as_bytes());
                res
            }
            ObjectValue::Object(v) => {
                let mut res = vec![5];
                res.extend(v.as_bytes());
                res
            }
            ObjectValue::Enum(type_id, variant, value) => {
                let mut res = vec![6];
                res.extend(type_id.as_bytes());
                res.extend(variant.as_bytes());
                res.extend(value.as_bytes());
                res
            }
        }
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            Some(next) => next,
            None => {
                return Err(VMError::runtime(format!(
                    "Missing byte for ObjectValue {location}"
                )))
            }
        };

        let v = match next {
            1 => ObjectValue::Primitive(Snapshot::from_bytes(bytes, location)?),
            2 => ObjectValue::List(Snapshot::from_bytes(bytes, location)?),
            3 => ObjectValue::Map(Snapshot::from_bytes(bytes, location)?),
            4 => ObjectValue::Tuple(Snapshot::from_bytes(bytes, location)?),
            5 => ObjectValue::Object(Snapshot::from_bytes(bytes, location)?),
            6 => ObjectValue::Enum(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            b => {
                return Err(VMError::runtime(format!(
                    "Illegal byte {b} for ObjectValue {location}"
                )))
            }
        };
        Ok(v)
    }
}
