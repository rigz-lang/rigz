use crate::lifecycle::Lifecycle;
use crate::{Instruction, Register};

/**
todo need to know whether scope is function, root, or expression for returns
for functions, return value
for root, this is a halt
otherwise inner scope returns should cascade to function call or root
*/

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope<'vm> {
    pub instructions: Vec<Instruction<'vm>>,
    pub lifecycle: Option<Lifecycle>,
    pub named: &'vm str,
    pub args: Vec<(&'vm str, bool)>,
}

impl Default for Scope<'_> {
    fn default() -> Self {
        Scope {
            named: "main",
            instructions: Default::default(),
            lifecycle: None,
            args: vec![],
        }
    }
}

impl<'vm> Scope<'vm> {
    #[inline]
    pub fn new(named: &'vm str, args: Vec<(&'vm str, bool)>) -> Self {
        Scope {
            named,
            args,
            ..Default::default()
        }
    }

    #[inline]
    pub fn lifecycle(named: &'vm str, args: Vec<(&'vm str, bool)>, lifecycle: Lifecycle) -> Self {
        Scope {
            lifecycle: Some(lifecycle),
            named,
            args,
            ..Default::default()
        }
    }
}
