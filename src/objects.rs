use crate::VMError;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Serialize, Deserialize)]
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
    Type {
        base_type: Box<RigzType>,
        optional: bool,
        can_return_error: bool,
    },
    Function(Vec<RigzType>, Box<RigzType>),
    Custom(CustomType),
}

impl Default for RigzType {
    fn default() -> Self {
        RigzType::Type {
            base_type: Box::new(RigzType::Any),
            optional: false,
            can_return_error: true,
        }
    }
}

impl RigzType {
    #[inline]
    pub fn is_vm(&self) -> bool {
        if let RigzType::Custom(c) = &self {
            if c.name.as_str() == "VM" {
                return true
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
            // todo lists & maps are [<Type>] & {<Type>, <Type>} or {<Type>}
            "List" => RigzType::List(Box::new(RigzType::Any)),
            "Map" => RigzType::Map(Box::new(RigzType::Any), Box::new(RigzType::Any)),
            "Range" => RigzType::Range,
            "String" => RigzType::String,
            s => {
                if let Some(s) = s.strip_suffix("!?") {
                    RigzType::Type {
                        base_type: Box::new(s.parse()?),
                        optional: true,
                        can_return_error: true,
                    }
                } else if let Some(s) = s.strip_suffix("!") {
                    RigzType::Type {
                        base_type: Box::new(s.parse()?),
                        optional: false,
                        can_return_error: true,
                    }
                } else if let Some(s) = s.strip_suffix("?") {
                    RigzType::Type {
                        base_type: Box::new(s.parse()?),
                        optional: true,
                        can_return_error: false,
                    }
                } else if s.contains("<") {
                    return Err(VMError::RuntimeError(
                        "Types containing < are not supported yet".to_string(),
                    ));
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
            RigzType::List(t) => write!(f, "List<{t}>"),
            RigzType::Map(k, v) => write!(f, "Map<{k},{v}>"),
            RigzType::Error => write!(f, "Error"),
            RigzType::This => write!(f, "Self"),
            RigzType::Range => write!(f, "Range"),
            RigzType::Type {
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
            RigzType::Custom(c) => write!(f, "{}", c.name),
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Serialize, Deserialize)]
pub struct CustomType {
    pub name: String,
    pub fields: Vec<(String, RigzType)>,
}

// todo create an object/class type and store type definitions in scope
