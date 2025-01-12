use crate::{Reference, Snapshot, StackValue, VMError};
use indexmap::map::Entry;
use indexmap::IndexMap;
use log_derive::{logfn, logfn_inputs};
use std::cell::RefCell;
use std::fmt::Display;
use std::ops::Index;
use std::vec::IntoIter;

#[derive(Clone, Debug)]
pub enum Variable {
    Let(StackValue),
    Mut(StackValue),
}

#[derive(Clone, Debug)]
pub struct Frames {
    pub current: RefCell<CallFrame>,
    pub frames: Vec<RefCell<CallFrame>>,
}

impl Snapshot for Frames {
    fn as_bytes(&self) -> Vec<u8> {
        let mut res = self.current.as_bytes();
        res.extend(self.frames.as_bytes());
        res
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        todo!()
    }
}

impl Index<usize> for Frames {
    type Output = RefCell<CallFrame>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.frames[index]
    }
}

impl Frames {
    #[inline]
    pub fn reset(&mut self) {
        self.current = RefCell::new(CallFrame::main());
        self.frames.clear();
    }

    #[inline]
    pub fn pop(&mut self) -> Option<RefCell<CallFrame>> {
        self.frames.pop()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    #[inline]
    pub fn push(&mut self, call_frame: CallFrame) {
        self.frames.push(call_frame.into())
    }

    #[inline]
    #[logfn_inputs(Trace, fmt = "load_let(frames={:#?} name={}, value={:?})")]
    pub fn load_let(&self, name: Reference<String>, value: StackValue) -> Result<(), VMError> {
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

    #[logfn(Trace)]
    #[logfn_inputs(Trace, fmt = "get_variable(frames={:#p} name={})")]
    pub fn get_variable(&self, name: &Reference<String>) -> Option<StackValue> {
        self.current.borrow().get_variable(name, self)
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace, fmt = "get_mutable_variable(frames={:#p} name={})")]
    pub fn get_mutable_variable(
        &self,
        name: &Reference<String>,
    ) -> Result<Option<StackValue>, VMError> {
        self.current.borrow().get_mutable_variable(name, self)
    }

    #[inline]
    #[logfn_inputs(Trace, fmt = "load_mut(frames={:#?} name={}, value={:?})")]
    pub fn load_mut(&self, name: Reference<String>, value: StackValue) -> Result<(), VMError> {
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

impl Default for Frames {
    #[inline]
    fn default() -> Self {
        Frames {
            current: RefCell::new(CallFrame::main()),
            frames: vec![],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CallFrame {
    pub scope_id: usize,
    pub pc: usize,
    pub variables: IndexMap<Reference<String>, Variable>,
    pub parent: Option<usize>,
}

impl Snapshot for CallFrame {
    fn as_bytes(&self) -> Vec<u8> {
        todo!()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        todo!()
    }
}

impl CallFrame {
    fn get_variable(&self, name: &Reference<String>, frames: &Frames) -> Option<StackValue> {
        match self.variables.get(name) {
            None => match self.parent {
                None => None,
                Some(parent) => frames[parent].borrow().get_variable(name, frames),
            },
            Some(v) => match v {
                Variable::Let(v) => Some(v.clone()),
                Variable::Mut(v) => Some(v.clone()),
            },
        }
    }

    fn get_mutable_variable(
        &self,
        name: &Reference<String>,
        frames: &Frames,
    ) -> Result<Option<StackValue>, VMError> {
        match self.variables.get(name) {
            None => match self.parent {
                None => Ok(None),
                Some(parent) => frames[parent].borrow().get_mutable_variable(name, frames),
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

impl CallFrame {
    #[inline]
    pub fn main() -> Self {
        Self::default()
    }

    #[inline]
    pub fn child(scope_id: usize, call_frame_id: usize) -> Self {
        Self {
            scope_id,
            parent: Some(call_frame_id),
            ..Default::default()
        }
    }
}
