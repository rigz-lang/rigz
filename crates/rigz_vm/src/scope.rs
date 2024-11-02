use crate::lifecycle::Lifecycle;
use crate::Instruction;

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
}

impl Default for Scope<'_> {
    fn default() -> Self {
        Scope {
            lifecycle: None,
            named: "main",
            instructions: Default::default(),
        }
    }
}

impl<'vm> Scope<'vm> {
    #[inline]
    pub fn new(named: &'vm str) -> Self {
        Scope {
            lifecycle: None,
            named,
            ..Default::default()
        }
    }

    #[inline]
    pub fn lifecycle(named: &'vm str, lifecycle: Lifecycle) -> Self {
        Scope {
            lifecycle: Some(lifecycle),
            named,
            ..Default::default()
        }
    }
}
