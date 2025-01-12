use crate::lifecycle::Lifecycle;
use crate::{Instruction, Reference, Snapshot, VMError};
use std::fmt::Display;
use std::vec::IntoIter;

/**
todo need to know whether scope is function, root, or expression for returns
for functions, return value
for root, this is a halt
otherwise inner scope returns should cascade to function call or root
*/

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope {
    pub instructions: Vec<Instruction>,
    pub lifecycle: Option<Lifecycle>,
    pub named: Reference<String>,
    pub args: Vec<(Reference<String>, bool)>,
    pub set_self: Option<bool>,
}

impl Default for Scope {
    fn default() -> Self {
        Scope {
            named: "main".to_string().into(),
            instructions: Default::default(),
            lifecycle: None,
            args: vec![],
            set_self: None,
        }
    }
}

impl Snapshot for Scope {
    fn as_bytes(&self) -> Vec<u8> {
        todo!()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        todo!()
    }
}

impl Scope {
    #[inline]
    pub fn new(
        named: Reference<String>,
        args: Vec<(Reference<String>, bool)>,
        set_self: Option<bool>,
    ) -> Self {
        Scope {
            named,
            args,
            set_self,
            ..Default::default()
        }
    }

    #[inline]
    pub fn lifecycle(
        named: Reference<String>,
        args: Vec<(Reference<String>, bool)>,
        lifecycle: Lifecycle,
        set_self: Option<bool>,
    ) -> Self {
        Scope {
            lifecycle: Some(lifecycle),
            named,
            args,
            set_self,
            ..Default::default()
        }
    }
}
