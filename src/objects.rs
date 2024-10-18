use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;
use crate::VMError;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
    VM,
    Range,
    Type(Box<RigzType>),
    Function(Vec<RigzType>, Box<RigzType>),
    Custom(CustomType),
}

impl FromStr for RigzType {
    type Err = VMError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rigz_type = match s {
            "None" => RigzType::None,
            "Bool" => RigzType::Float,
            "Float" => RigzType::Float,
            "Int" => RigzType::Int,
            "Number" => RigzType::Number,
            "Self" => RigzType::This,
            "Error" => RigzType::Error,
            "List" => RigzType::List(Box::new(RigzType::Any)),
            "Map" => RigzType::Map(Box::new(RigzType::Any), Box::new(RigzType::Any)),
            "Range" => RigzType::Range,
            "String" => RigzType::String,
            "VM" => RigzType::VM,
            s => {
                if s.contains("<") {
                    return Err(VMError::RuntimeError("Types containing < are not supported yet".to_string()))
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
            RigzType::VM => write!(f, "VM"),
            RigzType::Range => write!(f, "Range"),
            RigzType::Type(t) => write!(f, "Type<{t}>"),
            RigzType::Function(args, result) => write!(f, "Function<{args:?},{result}>"),
            RigzType::Custom(c) => write!(f, "{}", c.name),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomType {
    pub name: String,
    pub fields: Vec<(String, RigzType)>,
}

// todo create an object/class type and store type definitions in scope
