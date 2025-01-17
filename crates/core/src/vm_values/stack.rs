use crate::{ObjectValue, Snapshot, VMError};
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;
use std::vec::IntoIter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StackValue {
    ScopeId(usize),
    Value(Rc<RefCell<ObjectValue>>),
    Constant(usize),
}

impl<T: Into<ObjectValue>> From<T> for StackValue {
    fn from(value: T) -> Self {
        let v: ObjectValue = value.into();
        StackValue::Value(v.into())
    }
}

impl From<Rc<RefCell<ObjectValue>>> for StackValue {
    #[inline]
    fn from(value: Rc<RefCell<ObjectValue>>) -> Self {
        StackValue::Value(value)
    }
}

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

pub trait ResolveValue {
    fn location(&self) -> &'static str;

    fn handle_scope(&mut self, scope: usize) -> Rc<RefCell<ObjectValue>>;

    fn get_constant(&self, constant_id: usize) -> Rc<RefCell<ObjectValue>>;
}

impl StackValue {
    pub fn resolve<T: ResolveValue + ?Sized>(&self, vm: &mut T) -> Rc<RefCell<ObjectValue>> {
        match self {
            &StackValue::ScopeId(scope) => vm.handle_scope(scope),
            StackValue::Value(v) => v.clone(),
            &StackValue::Constant(c) => vm.get_constant(c),
        }
    }
}
