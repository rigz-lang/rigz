use crate::{Instruction, Register};


/**
todo need to know whether scope is function, root, or expression for returns
for functions, return value
for root, this is a halt
otherwise inner scope returns should cascade to function call or root
*/

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
