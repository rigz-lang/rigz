use log_derive::{logfn, logfn_inputs};
use rigz_core::{IndexMap, IndexMapEntry, Snapshot, StackValue, VMError};
use std::cell::RefCell;
use std::fmt::Display;
use std::ops::Index;
use std::vec::IntoIter;

#[derive(Clone, Debug, PartialEq)]
pub enum Variable {
    Let(StackValue),
    Mut(StackValue),
}

impl Snapshot for Variable {
    fn as_bytes(&self) -> Vec<u8> {
        let (mut res, v) = match self {
            Variable::Let(v) => (vec![0], v),
            Variable::Mut(v) => (vec![1], v),
        };
        res.extend(v.as_bytes());
        res
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        match bytes.next() {
            None => Err(VMError::runtime(format!(
                "Missing Variable byte - {location}"
            ))),
            Some(0) => Ok(Variable::Let(Snapshot::from_bytes(bytes, location)?)),
            Some(1) => Ok(Variable::Mut(Snapshot::from_bytes(bytes, location)?)),
            Some(b) => Err(VMError::runtime(format!(
                "Illegal Variable byte {b} - {location}"
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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
        let current = Snapshot::from_bytes(bytes, location)?;
        let frames = Snapshot::from_bytes(bytes, location)?;
        Ok(Frames { current, frames })
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
    #[logfn_inputs(Trace, fmt = "load_let(frames={:#?} name={}, value={:?}, shadow={})")]
    pub fn load_let(&self, name: String, value: StackValue, shadow: bool) -> Result<(), VMError> {
        match self.current.borrow_mut().variables.entry(name) {
            IndexMapEntry::Occupied(mut v) => {
                if shadow {
                    *v.get_mut() = Variable::Let(value);
                } else {
                    return Err(VMError::UnsupportedOperation(format!(
                        "Cannot overwrite let variable: {}",
                        *v.key()
                    )));
                }
            }
            IndexMapEntry::Vacant(e) => {
                e.insert(Variable::Let(value));
            }
        }
        Ok(())
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace, fmt = "get_variable(frames={:#p} name={})")]
    pub fn get_variable(&self, name: &str) -> Option<StackValue> {
        self.current.borrow().get_variable(name, self)
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace, fmt = "get_mutable_variable(frames={:#p} name={})")]
    pub fn get_mutable_variable(&self, name: &str) -> Result<Option<StackValue>, VMError> {
        self.current.borrow().get_mutable_variable(name, self)
    }

    #[inline]
    #[logfn_inputs(Trace, fmt = "load_mut(frames={:#?} name={}, value={:?}, shadow={})")]
    pub fn load_mut(&self, name: String, value: StackValue, shadow: bool) -> Result<(), VMError> {
        match self.current.borrow_mut().variables.entry(name) {
            IndexMapEntry::Occupied(mut var) => match var.get() {
                Variable::Let(_) => {
                    if shadow {
                        *var.get_mut() = Variable::Mut(value);
                    } else {
                        return Err(VMError::UnsupportedOperation(format!(
                            "Cannot overwrite let variable: {}",
                            *var.key()
                        )));
                    }
                }
                Variable::Mut(_) => {
                    *var.get_mut() = Variable::Mut(value);
                }
            },
            IndexMapEntry::Vacant(e) => {
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CallFrame {
    pub scope_id: usize,
    pub pc: usize,
    pub variables: IndexMap<String, Variable>,
    pub parent: Option<usize>,
}

impl Snapshot for CallFrame {
    fn as_bytes(&self) -> Vec<u8> {
        let mut res = self.scope_id.as_bytes();
        res.extend(self.pc.as_bytes());
        res.extend(self.variables.as_bytes());
        res.extend(self.parent.as_bytes());
        res
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let scope_id = Snapshot::from_bytes(bytes, location)?;
        let pc = Snapshot::from_bytes(bytes, location)?;
        let variables = Snapshot::from_bytes(bytes, location)?;
        let parent = Snapshot::from_bytes(bytes, location)?;
        Ok(CallFrame {
            scope_id,
            pc,
            variables,
            parent,
        })
    }
}

impl CallFrame {
    fn get_variable(&self, name: &str, frames: &Frames) -> Option<StackValue> {
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
        name: &str,
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

    pub(crate) fn clear_variables(&mut self) {
        self.variables.clear();
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
