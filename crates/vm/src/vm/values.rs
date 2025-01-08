use crate::vm::VM;
use crate::{Runner, VMError, Value};
use std::cell::RefCell;
use std::rc::Rc;

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

impl ResolveValue for VM<'_> {
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
