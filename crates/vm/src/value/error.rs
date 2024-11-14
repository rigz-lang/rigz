use crate::Value;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
// Tagged to avoid confusion with string deserialization
pub enum VMError {
    TimeoutError(String),
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
            VMError::TimeoutError(m) => write!(f, "Timeout Error: {m}"),
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
