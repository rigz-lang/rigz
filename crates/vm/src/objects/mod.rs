use crate::{Number, VMError, Value};
use dyn_clone::DynClone;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;

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
    Union(Vec<RigzType>),
    Composite(Vec<RigzType>),
    Custom(CustomType),
}

impl Default for RigzType {
    fn default() -> Self {
        RigzType::Type {
            base_type: Box::new(RigzType::Any),
            optional: true,
            can_return_error: true,
        }
    }
}

pub trait Object: Debug + Display + DynClone {
    fn name(&self) -> &'static str;

    fn from_map(index_map: IndexMap<Value, Value>) -> Result<Self, VMError>
    where
        Self: Sized;

    fn to_map(&self) -> IndexMap<Value, Value>;

    fn to_float(&self) -> Result<f64, VMError> {
        Err(VMError::ConversionError(format!(
            "Cannot convert {} to float",
            self.name()
        )))
    }
    fn to_int(&self) -> Result<i64, VMError> {
        Err(VMError::ConversionError(format!(
            "Cannot convert {} to int",
            self.name()
        )))
    }

    fn to_number(&self) -> Result<Number, VMError> {
        Err(VMError::ConversionError(format!(
            "Cannot convert {} to number",
            self.name()
        )))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NumberObject(pub Number);

impl Display for NumberObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Object for NumberObject {
    fn name(&self) -> &'static str {
        "Number"
    }

    fn from_map(index_map: IndexMap<Value, Value>) -> Result<Self, VMError>
    where
        Self: Sized,
    {
        match index_map.get(&Value::String("value".into())) {
            None => Err(VMError::ConversionError(format!(
                "Cannot convert map {:?} to Number Object",
                index_map
            ))),
            Some(v) => v.to_number().map(|n| NumberObject(n)),
        }
    }

    fn to_map(&self) -> IndexMap<Value, Value> {
        IndexMap::from([("value".into(), self.0.into())])
    }

    fn to_float(&self) -> Result<f64, VMError> {
        Ok(self.0.to_float())
    }

    fn to_int(&self) -> Result<i64, VMError> {
        Ok(self.0.to_int())
    }

    fn to_number(&self) -> Result<Number, VMError> {
        Ok(self.0)
    }
}

dyn_clone::clone_trait_object!(Object);

impl RigzType {
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

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Serialize, Deserialize)]
pub struct CustomType {
    pub name: String,
    pub fields: Vec<(String, RigzType)>,
}

// todo create an object/class type and store type definitions in scope
