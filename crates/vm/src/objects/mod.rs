use crate::{Snapshot, VMError};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;
use std::vec::IntoIter;

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Serialize, Deserialize)]
pub enum RigzType {
    None,
    Any,
    Bool,
    Int,
    Float,
    Number,
    String,
    List(Box<RigzType>),
    Map(Box<RigzType>, Box<RigzType>),
    Error,
    This,
    Range,
    Type,
    Wrapper {
        base_type: Box<RigzType>,
        optional: bool,
        can_return_error: bool,
    },
    Function(Vec<RigzType>, Box<RigzType>),
    Tuple(Vec<RigzType>),
    Union(Vec<RigzType>),
    Composite(Vec<RigzType>),
    Custom(CustomType),
}

impl Default for RigzType {
    fn default() -> Self {
        RigzType::Wrapper {
            base_type: Box::new(RigzType::Any),
            optional: true,
            can_return_error: true,
        }
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
            b => {
                return Err(VMError::RuntimeError(format!(
                    "Illegal RigzType byte {b} - {location}"
                )))
            }
        };
        Ok(rt)
    }
}

impl RigzType {
    pub fn matches(&self, other: &RigzType) -> bool {
        if self == other {
            return true;
        }

        matches!(self, RigzType::Any | RigzType::This)
    }

    #[inline]
    pub fn is_vm(&self) -> bool {
        if let RigzType::Custom(c) = &self {
            if c.name.as_str() == "VM" {
                return true;
            }
        }
        false
    }
}

impl FromStr for RigzType {
    type Err = VMError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rigz_type = match s {
            "None" => RigzType::None,
            "Any" => RigzType::Any,
            "Bool" => RigzType::Bool,
            "Float" => RigzType::Float,
            "Int" => RigzType::Int,
            "Number" => RigzType::Number,
            "Self" => RigzType::This,
            "Error" => RigzType::Error,
            // lists & maps can be [<Type>], {<Type>, <Type>}, or {<Type>}. This is handled within AST
            "List" => RigzType::List(Box::new(RigzType::Any)),
            "Map" => RigzType::Map(Box::new(RigzType::Any), Box::new(RigzType::Any)),
            "Range" => RigzType::Range,
            "String" => RigzType::String,
            "Type" => RigzType::Type,
            s => {
                if let Some(s) = s.strip_suffix("!?") {
                    RigzType::Wrapper {
                        base_type: Box::new(s.parse()?),
                        optional: true,
                        can_return_error: true,
                    }
                } else if let Some(s) = s.strip_suffix("!") {
                    RigzType::Wrapper {
                        base_type: Box::new(s.parse()?),
                        optional: false,
                        can_return_error: true,
                    }
                } else if let Some(s) = s.strip_suffix("?") {
                    RigzType::Wrapper {
                        base_type: Box::new(s.parse()?),
                        optional: true,
                        can_return_error: false,
                    }
                } else {
                    RigzType::Custom(CustomType {
                        name: s.to_string(),
                        fields: vec![],
                    })
                }
            }
        };
        Ok(rigz_type)
    }
}

impl Display for RigzType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RigzType::None => write!(f, "None"),
            RigzType::Any => write!(f, "Any"),
            RigzType::Bool => write!(f, "Bool"),
            RigzType::Int => write!(f, "Int"),
            RigzType::Float => write!(f, "Float"),
            RigzType::Number => write!(f, "Number"),
            RigzType::String => write!(f, "String"),
            RigzType::List(t) => write!(f, "[{t}]"),
            RigzType::Map(k, v) => write!(f, "{{{k},{v}}}"),
            RigzType::Error => write!(f, "Error"),
            RigzType::This => write!(f, "Self"),
            RigzType::Range => write!(f, "Range"),
            RigzType::Type => write!(f, "Type"),
            RigzType::Wrapper {
                base_type,
                optional,
                can_return_error,
            } => {
                write!(
                    f,
                    "{base_type}{}{}",
                    if *can_return_error { "!" } else { "" },
                    if *optional { "?" } else { "" }
                )
            }
            RigzType::Function(args, result) => write!(f, "Function<{args:?},{result}>"),
            RigzType::Tuple(args) => {
                write!(f, "({})", args.iter().map(|m| m.to_string()).join(" , "))
            }
            RigzType::Union(args) => {
                write!(f, "{}", args.iter().map(|m| m.to_string()).join(" | "))
            }
            RigzType::Composite(args) => {
                write!(f, "{}", args.iter().map(|m| m.to_string()).join(" & "))
            }
            RigzType::Custom(c) => write!(f, "{}", c.name),
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Serialize, Deserialize)]
pub struct CustomType {
    pub name: String,
    pub fields: Vec<(String, RigzType)>,
}

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

// todo create an object/class type and store type definitions in scope
