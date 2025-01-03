use crate::{Register, VMError, Value, VM};
use indexmap::map::Entry;
use indexmap::IndexMap;
use log_derive::{logfn, logfn_inputs};
use std::cell::RefCell;
use std::ops::Index;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Variable {
    Let(Rc<RefCell<Value>>),
    Mut(Rc<RefCell<Value>>),
}

#[derive(Clone, Debug)]
pub struct Frames<'vm> {
    pub current: RefCell<CallFrame<'vm>>,
    pub frames: Vec<RefCell<CallFrame<'vm>>>,
}

impl<'vm> Index<usize> for Frames<'vm> {
    type Output = RefCell<CallFrame<'vm>>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.frames[index]
    }
}

impl<'vm> Frames<'vm> {
    #[inline]
    pub fn pop(&mut self) -> Option<RefCell<CallFrame<'vm>>> {
        self.frames.pop()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    #[inline]
    pub fn push(&mut self, call_frame: CallFrame<'vm>) {
        self.frames.push(call_frame.into())
    }

    #[logfn_inputs(Trace, fmt = "load_let(frames={:#?} name={}, value={:?})")]
    pub fn load_let(&self, name: &'vm str, value: Rc<RefCell<Value>>) -> Result<(), VMError> {
        match self.current.borrow_mut().variables.entry(name) {
            Entry::Occupied(v) => {
                return Err(VMError::UnsupportedOperation(format!(
                    "Cannot overwrite let variable: {}",
                    *v.key()
                )))
            }
            Entry::Vacant(e) => {
                e.insert(Variable::Let(value));
            }
        }
        Ok(())
    }

    #[logfn_inputs(Trace, fmt = "load_mut(frames={:#?} name={}, value={:?})")]
    pub fn load_mut(&self, name: &'vm str, value: Rc<RefCell<Value>>) -> Result<(), VMError> {
        match self.current.borrow_mut().variables.entry(name) {
            Entry::Occupied(mut var) => match var.get() {
                Variable::Let(_) => {
                    return Err(VMError::UnsupportedOperation(format!(
                        "Cannot overwrite let variable: {}",
                        *var.key()
                    )))
                }
                Variable::Mut(_) => {
                    var.insert(Variable::Mut(value));
                }
            },
            Entry::Vacant(e) => {
                e.insert(Variable::Mut(value));
            }
        }
        Ok(())
    }
}

impl Default for Frames<'_> {
    fn default() -> Self {
        Frames {
            current: RefCell::new(CallFrame::main()),
            frames: vec![],
        }
    }
}

#[derive(Clone, Debug)]
pub struct CallFrame<'vm> {
    pub scope_id: usize,
    pub pc: usize,
    pub variables: IndexMap<&'vm str, Variable>,
    pub parent: Option<usize>,
    pub output: Register,
}

impl<'vm> CallFrame<'vm> {
    #[logfn(Trace)]
    #[logfn_inputs(Trace, fmt = "get_variable(frame={:#p} name={}, vm={:#p})")]
    pub(crate) fn get_variable(&self, name: &'vm str, vm: &VM<'vm>) -> Option<Rc<RefCell<Value>>> {
        match self.variables.get(name) {
            None => match self.parent {
                None => None,
                Some(parent) => vm.frames[parent].borrow().get_variable(name, vm),
            },
            Some(v) => match v {
                Variable::Let(v) => Some(v.clone()),
                Variable::Mut(v) => Some(v.clone()),
            },
        }
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace, fmt = "get_mutable_variable(frame={:#p} name={}, vm={:#p})")]
    pub(crate) fn get_mutable_variable(
        &self,
        name: &'vm str,
        vm: &VM<'vm>,
    ) -> Result<Option<Rc<RefCell<Value>>>, VMError> {
        match self.variables.get(name) {
            None => match self.parent {
                None => Ok(None),
                Some(parent) => vm.frames[parent].borrow().get_mutable_variable(name, vm),
            },
            Some(v) => match v {
                Variable::Let(_) => Err(VMError::VariableDoesNotExist(format!(
                    "Variable {} is immutable",
                    name
                ))),
                Variable::Mut(v) => Ok(Some(v.clone())),
            },
        }
    }
}

impl CallFrame<'_> {
    #[inline]
    pub fn main() -> Self {
        Self {
            output: 0,
            scope_id: 0,
            pc: 0,
            variables: Default::default(),
            parent: None,
        }
    }

    #[inline]
    pub fn child(scope_id: usize, call_frame_id: usize, output: Register) -> Self {
        Self {
            scope_id,
            output,
            pc: 0,
            variables: Default::default(),
            parent: Some(call_frame_id),
        }
    }
}
