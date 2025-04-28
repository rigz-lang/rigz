use crate::{CustomType, RigzType, Snapshot, VMError};
use std::fmt::Display;
use std::vec::IntoIter;

impl Snapshot for CustomType {
    fn as_bytes(&self) -> Vec<u8> {
        let mut res = Snapshot::as_bytes(&self.name);
        res.extend(self.fields.as_bytes());
        res
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        Ok(CustomType {
            name: Snapshot::from_bytes(bytes, location)?,
            fields: Snapshot::from_bytes(bytes, location)?,
        })
    }
}

impl Snapshot for RigzType {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            RigzType::None => vec![0],
            RigzType::Any => vec![1],
            RigzType::Bool => vec![2],
            RigzType::Int => vec![3],
            RigzType::Float => vec![4],
            RigzType::Number => vec![5],
            RigzType::String => vec![6],
            RigzType::List(v) => {
                let mut res = vec![7];
                res.extend(v.as_bytes());
                res
            }
            RigzType::Map(k, v) => {
                let mut res = vec![8];
                res.extend(k.as_bytes());
                res.extend(v.as_bytes());
                res
            }
            RigzType::Error => vec![9],
            RigzType::This => vec![10],
            RigzType::Range => vec![11],
            RigzType::Type => vec![12],
            RigzType::Wrapper {
                base_type,
                optional,
                can_return_error,
            } => {
                let mut res = vec![13];
                res.extend(base_type.as_bytes());
                res.extend(optional.as_bytes());
                res.extend(can_return_error.as_bytes());
                res
            }
            RigzType::Function(a, r) => {
                let mut res = vec![14];
                res.extend(a.as_bytes());
                res.extend(r.as_bytes());
                res
            }
            RigzType::Tuple(v) => {
                let mut res = vec![15];
                res.extend(v.as_bytes());
                res
            }
            RigzType::Union(v) => {
                let mut res = vec![16];
                res.extend(v.as_bytes());
                res
            }
            RigzType::Composite(v) => {
                let mut res = vec![17];
                res.extend(v.as_bytes());
                res
            }
            RigzType::Custom(c) => {
                let mut res = vec![18];
                res.extend(c.as_bytes());
                res
            }
            RigzType::Enum(c) => {
                let mut res = vec![19];
                res.extend(c.as_bytes());
                res
            }
        }
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            Some(b) => b,
            None => {
                return Err(VMError::RuntimeError(format!(
                    "Missing RigzType byte {location}"
                )))
            }
        };

        let rt = match next {
            0 => RigzType::None,
            1 => RigzType::Any,
            2 => RigzType::Bool,
            3 => RigzType::Int,
            4 => RigzType::Float,
            5 => RigzType::Number,
            6 => RigzType::String,
            7 => RigzType::List(Snapshot::from_bytes(bytes, location)?),
            8 => RigzType::Map(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            9 => RigzType::Error,
            10 => RigzType::This,
            11 => RigzType::Range,
            12 => RigzType::Type,
            13 => RigzType::Wrapper {
                base_type: Snapshot::from_bytes(bytes, location)?,
                optional: Snapshot::from_bytes(bytes, location)?,
                can_return_error: Snapshot::from_bytes(bytes, location)?,
            },
            14 => RigzType::Function(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            15 => RigzType::Tuple(Snapshot::from_bytes(bytes, location)?),
            16 => RigzType::Composite(Snapshot::from_bytes(bytes, location)?),
            17 => RigzType::Union(Snapshot::from_bytes(bytes, location)?),
            18 => RigzType::Custom(Snapshot::from_bytes(bytes, location)?),
            19 => RigzType::Enum(Snapshot::from_bytes(bytes, location)?),
            b => {
                return Err(VMError::RuntimeError(format!(
                    "Illegal RigzType byte {b} - {location}"
                )))
            }
        };
        Ok(rt)
    }
}
