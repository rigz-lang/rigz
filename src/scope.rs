use crate::{Instruction, Register, RigzObjectDefinition};
use indexmap::IndexMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scope<'vm> {
    pub instructions: Vec<Instruction<'vm>>,
    pub type_definitions: IndexMap<String, RigzObjectDefinition>,
    pub owned_registers: Vec<Register>
}

impl<'vm> Scope<'vm> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}
