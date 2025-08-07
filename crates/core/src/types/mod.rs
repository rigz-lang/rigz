#[cfg(feature = "snapshot")]
mod snapshot;

use crate::VMError;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Serialize, Deserialize)]
pub enum RigzType {
    None,
    Never,
    Any,
    Bool,
    Int,
    Float,
    Number,
    String,
    List(Box<RigzType>),
    Set(Box<RigzType>),
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
    Enum(usize),
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

impl RigzType {
    pub fn matches(&self, other: &RigzType) -> bool {
        if self == other {
            return true;
        }

        if matches!(self, RigzType::List(_)) && matches!(other, RigzType::List(_)) {
            return true;
        }

        if matches!(self, RigzType::Map(_, _)) && matches!(other, RigzType::Map(_, _)) {
            return true;
        }

        let s = if let RigzType::Wrapper { base_type, .. } = other {
            base_type.as_ref()
        } else {
            other
        };

        if self == s {
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
            "Never" => RigzType::Never,
            "Any" => RigzType::Any,
            "Bool" => RigzType::Bool,
            "Float" => RigzType::Float,
            "Int" => RigzType::Int,
            "Number" => RigzType::Number,
            "Self" => RigzType::This,
            "Error" => RigzType::Error,
            // lists & maps can be [<Type>], {<Type>, <Type>}, or {<Type>}. This is handled within AST
            "List" => RigzType::List(Box::new(RigzType::Any)),
            "Set" => RigzType::Set(Box::new(RigzType::Any)),
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
            RigzType::Never => write!(f, "Never"),
            RigzType::None => write!(f, "None"),
            RigzType::Any => write!(f, "Any"),
            RigzType::Bool => write!(f, "Bool"),
            RigzType::Int => write!(f, "Int"),
            RigzType::Float => write!(f, "Float"),
            RigzType::Number => write!(f, "Number"),
            RigzType::String => write!(f, "String"),
            RigzType::Set(t) => write!(f, "Set[{t}]"),
            RigzType::List(t) => write!(f, "[{t}]"),
            RigzType::Map(k, v) => write!(f, "{{{k},{v}}}"),
            RigzType::Error => write!(f, "Error"),
            RigzType::This => write!(f, "Self"),
            RigzType::Range => write!(f, "Range"),
            RigzType::Type => write!(f, "Type"),
            RigzType::Enum(i) => write!(f, "Enum {i}"),
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

#[derive(Clone, Debug, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub struct CustomType {
    pub name: String,
    pub fields: Vec<(String, RigzType)>,
}

impl Hash for CustomType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl PartialEq for CustomType {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}
