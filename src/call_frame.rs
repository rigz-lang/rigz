use std::cell::RefCell;
use crate::{Register, RegisterValue, VMError, VM};
use indexmap::{IndexMap, IndexSet};
use log_derive::{logfn, logfn_inputs};
use nohash_hasher::BuildNoHashHasher;

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
    pub stack: Vec<RegisterValue>,
    pub variables: IndexMap<&'vm str, Variable>, // TODO switch to intern strings
    pub parent: Option<usize>,
    pub owned_registers: IndexSet<Register, BuildNoHashHasher<Register>>,
    pub output: Register,
}

impl<'vm> CallFrame<'vm> {
    #[logfn(Trace)]
    #[logfn_inputs(Trace, fmt = "get_variable(frame={:#p} name={}, vm={:#p})")]
    pub(crate) fn get_variable(&self, name: &'vm str, vm: &VM<'vm>) -> Option<Register> {
        match self.variables.get(name) {
            None => match self.parent {
                None => None,
                Some(parent) => vm.frames[parent].get_variable(name, vm),
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
                Some(parent) => vm.frames[parent].get_mutable_variable(name, vm),
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
            stack: Default::default(),
            variables: Default::default(),
            owned_registers: Default::default(),
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
            stack: Default::default(),
            variables: Default::default(),
            owned_registers: Default::default(),
            parent: Some(parent),
        }
    }
}
