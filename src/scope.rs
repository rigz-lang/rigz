use indexmap::IndexMap;
use crate::{Instruction, Register, RigzObjectDefinition, VMError, Value, VM};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scope<'vm> {
    pub instructions: Vec<Instruction<'vm>>,
    pub type_definitions: IndexMap<String, RigzObjectDefinition>,
}

impl<'vm> Scope<'vm> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}