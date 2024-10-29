extern crate core;

mod builder;
mod call_frame;
mod instructions;
mod lifecycle;
mod macros;
mod module;
mod number;
mod objects;
mod scope;
mod traits;
mod value;
mod value_range;
mod vm;

#[cfg(feature = "derive")]
pub mod derive;

pub use builder::{RigzBuilder, VMBuilder};
pub use call_frame::{CallFrame, Variable};
pub use indexmap::map::Entry;
pub use indexmap::IndexMap;
pub use instructions::{
    Binary, BinaryAssign, BinaryOperation, Clear, Instruction, Unary, UnaryAssign, UnaryOperation,
};
pub use lifecycle::*;
pub use module::{Module, RigzArgs};
pub use number::Number;
pub use objects::{CustomType, RigzType};
pub use scope::Scope;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
pub use traits::{Logical, Reverse};
pub use value::Value;
pub use value_range::ValueRange;
pub use vm::{RegisterValue, VM};

pub type Register = usize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
// Tagged to avoid confusion with string deserialization
pub enum VMError {
    RuntimeError(String),
    EmptyRegister(String),
    ConversionError(String),
    ScopeDoesNotExist(String),
    UnsupportedOperation(String),
    VariableDoesNotExist(String),
    InvalidModule(String),
    InvalidModuleFunction(String),
    LifecycleError(String),
}

impl Display for VMError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VMError::RuntimeError(m) => write!(f, "{m}"),
            VMError::EmptyRegister(m) => write!(f, "Empty Register: {m}"),
            VMError::ConversionError(m) => write!(f, "Conversion Error: {m}"),
            VMError::ScopeDoesNotExist(m) => write!(f, "Scope Does Not Exist: {m}"),
            VMError::UnsupportedOperation(m) => write!(f, "Unsupported Operation: {m}"),
            VMError::VariableDoesNotExist(m) => write!(f, "Variable Does Not Exist: {m}"),
            VMError::InvalidModule(m) => write!(f, "Invalid Module: {m}"),
            VMError::InvalidModuleFunction(m) => write!(f, "Invalid Module Function: {m}"),
            VMError::LifecycleError(m) => write!(f, "Lifecycle Error: {m}"),
        }
    }
}

impl VMError {
    pub fn to_value(self) -> Value {
        Value::Error(self)
    }

    pub fn invalid_function(func: &str) -> Self {
        VMError::InvalidModuleFunction(format!("Function {func} does not exist"))
    }
}
