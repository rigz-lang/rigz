use std::fmt::{Display, Formatter};
use std::hash::Hash;

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
            RigzType::Custom(c) => write!(f, "Custom<{c:?}>"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomType {
    pub name: String,
    pub fields: Vec<(String, RigzType)>,
}

// todo create an object/class type and store type definitions in scope
