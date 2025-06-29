use crate::{Number, PrimitiveValue, Snapshot, VMError};
use itertools::Itertools;
use std::fmt::Display;
use std::vec::IntoIter;

impl Snapshot for PrimitiveValue {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            PrimitiveValue::None => vec![0],
            PrimitiveValue::Bool(b) => {
                let mut res = vec![1];
                res.extend(b.as_bytes());
                res
            }
            PrimitiveValue::Number(n) => {
                let mut res = match n {
                    Number::Int(_) => vec![2],
                    Number::Float(_) => vec![3],
                };
                res.extend(n.to_bytes());
                res
            }
            PrimitiveValue::String(s) => {
                let mut res = vec![4];
                res.extend(s.as_bytes());
                res
            }
            PrimitiveValue::Range(r) => {
                let mut res = vec![5];
                res.extend(r.as_bytes());
                res
            }
            PrimitiveValue::Error(e) => {
                let mut res = vec![6];
                res.extend(e.as_bytes());
                res
            }
            PrimitiveValue::Type(t) => {
                let mut res = vec![7];
                res.extend(t.as_bytes());
                res
            }
        }
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            None => return Err(VMError::runtime(format!("Missing Value byte {location}"))),
            Some(s) => s,
        };
        let v = match next {
            0 => PrimitiveValue::None,
            1 => PrimitiveValue::Bool(bool::from_bytes(bytes, location)?),
            2 => {
                let b = match bytes.next_array() {
                    None => {
                        return Err(VMError::runtime(format!(
                            "Missing Number::Int bytes {location}"
                        )))
                    }
                    Some(s) => s,
                };
                PrimitiveValue::Number(Number::Int(i64::from_be_bytes(b)))
            }
            3 => {
                let b = match bytes.next_array() {
                    None => {
                        return Err(VMError::runtime(format!(
                            "Missing Number::Float bytes {location}"
                        )))
                    }
                    Some(s) => s,
                };
                PrimitiveValue::Number(Number::Float(f64::from_be_bytes(b)))
            }
            4 => PrimitiveValue::String(Snapshot::from_bytes(bytes, location)?),
            5 => PrimitiveValue::Range(Snapshot::from_bytes(bytes, location)?),
            6 => PrimitiveValue::Error(Snapshot::from_bytes(bytes, location)?),
            7 => PrimitiveValue::Type(Snapshot::from_bytes(bytes, location)?),
            b => {
                return Err(VMError::runtime(format!(
                    "Illegal Value byte {b} - {location}"
                )))
            }
        };
        Ok(v)
    }
}
