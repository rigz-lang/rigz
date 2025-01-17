use crate::{ObjectValue, Snapshot, StackValue, VMError};
use std::fmt::Display;
use std::vec::IntoIter;

impl Snapshot for StackValue {
    fn as_bytes(&self) -> Vec<u8> {
        let mut results = Vec::new();
        match self {
            StackValue::ScopeId(s) => {
                results.push(0);
                results.extend(s.as_bytes());
            }
            StackValue::Value(v) => {
                results.push(1);
                results.extend(v.borrow().as_bytes());
            }
            StackValue::Constant(c) => {
                results.push(2);
                results.extend(c.as_bytes());
            }
        }
        results
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let tv = match bytes.next() {
            None => return Err(VMError::RuntimeError(format!("{location} StackValue type"))),
            Some(b) => b,
        };
        let l = match tv {
            0 => StackValue::ScopeId(Snapshot::from_bytes(bytes, location)?),
            1 => {
                let v: ObjectValue = Snapshot::from_bytes(bytes, location)?;
                StackValue::Value(v.into())
            }
            2 => StackValue::Constant(Snapshot::from_bytes(bytes, location)?),
            _ => {
                return Err(VMError::RuntimeError(format!(
                    "{location} Invalid StackValue type {tv}"
                )))
            }
        };
        Ok(l)
    }
}
