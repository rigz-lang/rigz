use crate::{ObjectValue, PrimitiveValue};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum VMError {
    TimeoutError(String),
    RuntimeError(Box<ObjectValue>),
    EmptyStack(String),
    ConversionError(String),
    ScopeDoesNotExist(String),
    UnsupportedOperation(String),
    VariableDoesNotExist(String),
    InvalidModule(String),
    InvalidModuleFunction(String),
    LifecycleError(String),
}

impl Error for VMError {}

impl From<VMError> for Rc<RefCell<PrimitiveValue>> {
    #[inline]
    fn from(value: VMError) -> Self {
        Rc::new(RefCell::new(value.into()))
    }
}

impl From<&VMError> for PrimitiveValue {
    #[inline]
    fn from(value: &VMError) -> Self {
        value.clone().into()
    }
}

impl Display for VMError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VMError::RuntimeError(m) => write!(f, "Error: {m}"),
            VMError::EmptyStack(m) => write!(f, "Empty Register: {m}"),
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
    pub fn runtime(value: String) -> Self {
        VMError::RuntimeError(Box::new(value.into()))
    }

    pub fn to_value(self) -> PrimitiveValue {
        PrimitiveValue::Error(self)
    }

    pub fn extract_value(self) -> ObjectValue {
        let str = match self {
            VMError::TimeoutError(s) => s,
            VMError::RuntimeError(s) => return *s,
            VMError::EmptyStack(s) => s,
            VMError::ConversionError(s) => s,
            VMError::ScopeDoesNotExist(s) => s,
            VMError::UnsupportedOperation(s) => s,
            VMError::VariableDoesNotExist(s) => s,
            VMError::InvalidModule(s) => s,
            VMError::InvalidModuleFunction(s) => s,
            VMError::LifecycleError(s) => s,
        };
        ObjectValue::Primitive(PrimitiveValue::String(str))
    }

    pub fn invalid_function(func: &str) -> Self {
        VMError::InvalidModuleFunction(format!("Function {func} does not exist"))
    }

    pub fn todo<T: Display>(message: T) -> Self {
        VMError::UnsupportedOperation(format!("Not implemented - {message}"))
    }
}
