use crate::{Register, RegisterValue, VMError, Value, VM};
use indexmap::IndexMap;
use log::warn;
use log_derive::{logfn, logfn_inputs};
use nohash_hasher::BuildNoHashHasher;
use std::cell::RefCell;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub enum Variable {
    Let(Register),
    Mut(Register),
}

#[derive(Clone, Debug)]
pub struct CallFrame<'vm> {
    pub scope_id: usize,
    pub pc: usize,
    pub registers: IndexMap<Register, RefCell<RegisterValue>, BuildNoHashHasher<Register>>,
    pub variables: IndexMap<&'vm str, Variable>,
    pub parent: Option<usize>,
    pub output: Register,
}

impl<'vm> CallFrame<'vm> {
    #[inline]
    pub(crate) fn remove_register(&mut self, register: Register, vm: &VM) -> RegisterValue {
        match self.registers.get_mut(&register) {
            None => match self.parent {
                None => RegisterValue::Value(
                    VMError::EmptyRegister(format!("R{} is empty (remove)", register)).to_value(),
                ),
                Some(i) => vm.frames[i].borrow_mut().remove_register(register, vm),
            },
            Some(v) => RefCell::replace(v, RegisterValue::Value(Value::None)),
        }
    }

    pub(crate) fn get_register(&self, register: Register, vm: &VM) -> RegisterValue {
        match self.registers.get(&register) {
            None => match self.parent {
                None => VMError::EmptyRegister(format!("R{} is empty", register)).into(),
                Some(s) => vm.frames[s].borrow().get_register(register, vm),
            },
            Some(v) => v.borrow().clone(),
        }
    }

    pub(crate) fn swap_register(&mut self, original: Register, dest: Register, vm: &VM) {
        if original == dest {
            warn!("Called swap_register with same register {dest}");
            return;
        }

        let res = self
            .registers
            .insert(original, RegisterValue::Register(dest).into());
        match res {
            None => match self.parent {
                None => {
                    self.registers.insert(
                        dest,
                        RefCell::new(
                            VMError::EmptyRegister(format!(
                                "Invalid call to swap_register {original} was not set"
                            ))
                            .into(),
                        ),
                    );
                }
                Some(i) => {
                    let v = vm.frames[i].borrow_mut().remove_register(original, vm);
                    self.registers.insert(dest, v.into());
                }
            },
            Some(res) => {
                {
                    let b = res.borrow();
                    if let RegisterValue::Register(r) = b.deref() {
                        return self.swap_register(*r, dest, vm);
                    }
                }
                self.registers.insert(dest, res);
            }
        }
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace, fmt = "get_variable(frame={:#p} name={}, vm={:#p})")]
    pub(crate) fn get_variable(&self, name: &'vm str, vm: &VM<'vm>) -> Option<Register> {
        match self.variables.get(name) {
            None => match self.parent {
                None => None,
                Some(parent) => vm.frames[parent].borrow().get_variable(name, vm),
            },
            Some(v) => match v {
                Variable::Let(v) => Some(*v),
                Variable::Mut(v) => Some(*v),
            },
        }
    }

    #[logfn(Trace)]
    #[logfn_inputs(Trace, fmt = "get_mutable_variable(frame={:#p} name={}, vm={:#p})")]
    pub(crate) fn get_mutable_variable(
        &self,
        name: &'vm str,
        vm: &VM<'vm>,
    ) -> Result<Option<Register>, VMError> {
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
                Variable::Mut(v) => Ok(Some(*v)),
            },
        }
    }
}

impl<'vm> CallFrame<'vm> {
    #[inline]
    pub fn main() -> Self {
        Self {
            output: 0,
            scope_id: 0,
            pc: 0,
            registers: Default::default(),
            variables: Default::default(),
            parent: None,
        }
    }

    #[inline]
    pub fn child(scope_id: usize, parent: usize, output: Register) -> Self {
        Self {
            scope_id,
            output,
            pc: 0,
            registers: Default::default(),
            variables: Default::default(),
            parent: Some(parent),
        }
    }
}
