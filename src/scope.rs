use crate::{Instruction, Register};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scope<'vm> {
    pub instructions: Vec<Instruction<'vm>>,
    pub owned_registers: Vec<Register>,
}

impl<'vm> Scope<'vm> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}
