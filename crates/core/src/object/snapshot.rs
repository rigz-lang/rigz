use crate::{ObjectValue, Snapshot, VMError};
use std::fmt::Display;
use std::vec::IntoIter;

macro_rules! object_snap {
    ($ty: tt, $name: literal) => {
        impl Snapshot for $ty {
            fn as_bytes(&self) -> Vec<u8> {
                match self {
                    $ty::Primitive(p) => {
                        let mut res = vec![1];
                        res.extend(p.as_bytes());
                        res
                    }
                    $ty::List(v) => {
                        let mut res = vec![2];
                        res.extend(v.as_bytes());
                        res
                    }
                    $ty::Map(m) => {
                        let mut res = vec![3];
                        res.extend(m.as_bytes());
                        res
                    }
                    $ty::Tuple(v) => {
                        let mut res = vec![4];
                        res.extend(v.as_bytes());
                        res
                    }
                    $ty::Object(v) => {
                        let mut res = vec![5];
                        res.extend(v.as_bytes());
                        res
                    }
                    $ty::Enum(type_id, variant, value) => {
                        let mut res = vec![6];
                        res.extend(type_id.as_bytes());
                        res.extend(variant.as_bytes());
                        res.extend(value.as_bytes());
                        res
                    }
                    $ty::Set(v) => {
                        let mut res = vec![7];
                        res.extend(v.as_bytes());
                        res
                    }
                }
            }

            fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
                let next = match bytes.next() {
                    Some(next) => next,
                    None => {
                        return Err(VMError::runtime(format!(
                            "Missing byte for {} {location}", $name
                        )))
                    }
                };

                let v = match next {
                    1 => $ty::Primitive(Snapshot::from_bytes(bytes, location)?),
                    2 => $ty::List(Snapshot::from_bytes(bytes, location)?),
                    3 => $ty::Map(Snapshot::from_bytes(bytes, location)?),
                    4 => $ty::Tuple(Snapshot::from_bytes(bytes, location)?),
                    5 => $ty::Object(Snapshot::from_bytes(bytes, location)?),
                    6 => $ty::Enum(
                        Snapshot::from_bytes(bytes, location)?,
                        Snapshot::from_bytes(bytes, location)?,
                        Snapshot::from_bytes(bytes, location)?,
                    ),
                    7 => $ty::Set(Snapshot::from_bytes(bytes, location)?),
                    b => {
                        return Err(VMError::runtime(format!("Illegal byte {b} for {} {location}", $name)))
                    }
                };
                Ok(v)
            }
        }
    };
}

object_snap!(ObjectValue, "ObjectValue");