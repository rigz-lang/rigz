use crate::{Register, VMError, VM};
use indexmap::IndexMap;

#[derive(Clone, Debug)]
pub enum Variable {
    Let(Register),
    Mut(Register),
}

#[derive(Clone, Debug)]
pub struct CallFrame<'vm> {
    pub scope_id: usize,
    pub pc: usize,
    pub variables: IndexMap<&'vm str, Variable>, // TODO switch to intern strings
    pub parent: Option<usize>,
    pub output: Register,
}

impl<'vm> CallFrame<'vm> {
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
            variables: Default::default(),
            parent: Some(parent),
        }
    }
}
