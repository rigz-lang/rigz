use crate::vm::VM;
use crate::{Runner, Snapshot, VMError, Value};
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;
use std::vec::IntoIter;

pub enum VMState {
    Running,
    Done(Rc<RefCell<Value>>),
    Ran(Rc<RefCell<Value>>),
}

impl From<VMError> for VMState {
    #[inline]
    fn from(value: VMError) -> Self {
        VMState::Done(value.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StackValue {
    ScopeId(usize),
    Value(Rc<RefCell<Value>>),
    Constant(usize),
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
                let v: Value = Snapshot::from_bytes(bytes, location)?;
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

impl From<Rc<RefCell<Value>>> for StackValue {
    #[inline]
    fn from(value: Rc<RefCell<Value>>) -> Self {
        StackValue::Value(value)
    }
}

pub trait ResolveValue {
    fn location(&self) -> &'static str;

    fn handle_scope(&mut self, scope: usize) -> Rc<RefCell<Value>>;

    fn get_constant(&self, constant_id: usize) -> Rc<RefCell<Value>>;
}

impl ResolveValue for VM {
    fn location(&self) -> &'static str {
        "VM"
    }

    #[inline]
    fn handle_scope(&mut self, scope: usize) -> Rc<RefCell<Value>> {
        let current = self.sp;
        match self.call_frame(scope) {
            Ok(_) => {}
            Err(e) => return e.into(),
        };
        let mut v = match self.run_scope() {
            VMState::Running => unreachable!(),
            VMState::Done(v) => return v,
            VMState::Ran(v) => v,
        };
        while current != self.sp {
            self.stack.push(v.into());
            v = match self.run_scope() {
                VMState::Running => unreachable!(),
                VMState::Done(v) => return v,
                VMState::Ran(v) => v,
            };
        }
        v
    }

    fn get_constant(&self, index: usize) -> Rc<RefCell<Value>> {
        match self.constants.get(index) {
            None => VMError::RuntimeError(format!("Constant {index} does not exist")).into(),
            Some(v) => v.clone().into(),
        }
    }
}

impl StackValue {
    pub fn resolve<T: ResolveValue + ?Sized>(&self, vm: &mut T) -> Rc<RefCell<Value>> {
        match self {
            &StackValue::ScopeId(scope) => vm.handle_scope(scope),
            StackValue::Value(v) => v.clone(),
            &StackValue::Constant(c) => vm.get_constant(c),
        }
    }
}

impl<T: Into<Value>> From<T> for StackValue {
    #[inline]
    fn from(value: T) -> Self {
        StackValue::Value(Rc::new(RefCell::new(value.into())))
    }
}
