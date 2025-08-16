use crate::Instruction;
use rigz_core::{Lifecycle, Snapshot, VMError};
use std::fmt::Display;
use std::vec::IntoIter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope {
    pub instructions: Vec<Instruction>,
    pub lifecycle: Option<Lifecycle>,
    pub named: String,
    pub args: Vec<(usize, bool)>,
    pub set_self: Option<bool>,
    pub propagate: bool
}

impl Default for Scope {
    fn default() -> Self {
        Scope {
            named: "main".to_string(),
            instructions: Default::default(),
            lifecycle: None,
            args: vec![],
            set_self: None,
            propagate: false,
        }
    }
}

impl Snapshot for Scope {
    fn as_bytes(&self) -> Vec<u8> {
        let mut res = Snapshot::as_bytes(&self.named);
        res.extend(self.instructions.as_bytes());
        res.extend(self.lifecycle.as_bytes());
        res.extend(self.args.as_bytes());
        res.extend(self.set_self.as_bytes());
        res.extend(self.propagate.as_bytes());
        res
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let named = String::from_bytes(bytes, location)?;
        let instructions = Snapshot::from_bytes(bytes, location)?;
        let lifecycle = Snapshot::from_bytes(bytes, location)?;
        let args = Snapshot::from_bytes(bytes, location)?;
        let set_self = Snapshot::from_bytes(bytes, location)?;
        let propagate = Snapshot::from_bytes(bytes, location)?;
        Ok(Scope {
            instructions,
            lifecycle,
            named,
            args,
            set_self,
            propagate
        })
    }
}

impl Scope {
    #[inline]
    pub fn new(named: String, args: Vec<(usize, bool)>, set_self: Option<bool>) -> Self {
        let propagate = matches!(named.as_str(), "if" | "unless" | "else" | "loop" | "for");
        Scope {
            named,
            args,
            set_self,
            propagate,
            ..Default::default()
        }
    }

    #[inline]
    pub fn lifecycle(
        named: String,
        args: Vec<(usize, bool)>,
        lifecycle: Lifecycle,
        set_self: Option<bool>,
    ) -> Self {
        let propagate = matches!(named.as_str(), "if" | "unless" | "else" | "loop" | "for");
        Scope {
            lifecycle: Some(lifecycle),
            named,
            args,
            set_self,
            propagate,
            ..Default::default()
        }
    }
}
