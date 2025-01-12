use crate::{Snapshot, Value};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::vec::IntoIter;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
// Tagged to avoid confusion with string deserialization
pub enum VMError {
    TimeoutError(String),
    RuntimeError(String),
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

impl Snapshot for VMError {
    fn as_bytes(&self) -> Vec<u8> {
        todo!()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        todo!()
    }
}

#[cfg(feature = "threaded")]
impl From<crossbeam::channel::RecvError> for VMError {
    fn from(value: crossbeam::channel::RecvError) -> Self {
        VMError::RuntimeError(format!("Process failed: {value:?}"))
    }
}

impl From<VMError> for Rc<RefCell<Value>> {
    #[inline]
    fn from(value: VMError) -> Self {
        Rc::new(RefCell::new(value.into()))
    }
}

impl From<&VMError> for Value {
    #[inline]
    fn from(value: &VMError) -> Self {
        value.clone().into()
    }
}

impl Display for VMError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VMError::RuntimeError(m) => write!(f, "{m}"),
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
    pub fn to_value(self) -> Value {
        Value::Error(self)
    }

    pub fn invalid_function(func: &str) -> Self {
        VMError::InvalidModuleFunction(format!("Function {func} does not exist"))
    }

    pub fn todo<T: Display>(message: T) -> Self {
        VMError::UnsupportedOperation(format!("Not implemented - {message}"))
    }
}
