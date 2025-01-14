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
        match self {
            VMError::TimeoutError(m) => {
                let mut res = vec![0];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::RuntimeError(m) => {
                let mut res = vec![1];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::EmptyStack(m) => {
                let mut res = vec![2];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::ConversionError(m) => {
                let mut res = vec![3];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::ScopeDoesNotExist(m) => {
                let mut res = vec![4];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::UnsupportedOperation(m) => {
                let mut res = vec![5];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::VariableDoesNotExist(m) => {
                let mut res = vec![6];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::InvalidModule(m) => {
                let mut res = vec![7];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::InvalidModuleFunction(m) => {
                let mut res = vec![8];
                res.extend(Snapshot::as_bytes(m));
                res
            }
            VMError::LifecycleError(m) => {
                let mut res = vec![9];
                res.extend(Snapshot::as_bytes(m));
                res
            }
        }
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            Some(s) => s,
            None => return Err(VMError::RuntimeError(format!("Missing VMError {location}"))),
        };
        let message = String::from_bytes(bytes, &format!("VMError - {location}"))?;
        let e = match next {
            0 => VMError::TimeoutError(message),
            1 => VMError::RuntimeError(message),
            2 => VMError::EmptyStack(message),
            3 => VMError::ConversionError(message),
            4 => VMError::ScopeDoesNotExist(message),
            5 => VMError::UnsupportedOperation(message),
            6 => VMError::VariableDoesNotExist(message),
            7 => VMError::InvalidModule(message),
            8 => VMError::InvalidModuleFunction(message),
            9 => VMError::LifecycleError(message),
            b => {
                return Err(VMError::RuntimeError(format!(
                    "Illegal VMError byte {b} {location}"
                )))
            }
        };
        Ok(e)
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
