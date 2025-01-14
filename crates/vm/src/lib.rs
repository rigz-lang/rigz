extern crate core;

mod builder;
mod call_frame;
mod instructions;
mod lifecycle;
mod macros;
mod module;
mod number;
mod objects;
mod operations;
mod scope;
mod traits;
mod value;
mod value_range;
mod vm;

#[cfg(feature = "derive")]
pub mod derive;
mod process;
mod stack;

pub type IndexMapEntry<'a, K, V> = indexmap::map::Entry<'a, K, V>;

pub use builder::{RigzBuilder, VMBuilder};
pub use call_frame::{CallFrame, Variable};
pub use indexmap::IndexMap;
pub use instructions::*;
pub use lifecycle::*;
pub use module::*;
pub use number::*;
pub use objects::*;
pub use operations::*;
pub use process::{ModulesMap, Reference};
pub use scope::Scope;
pub use stack::VMStack;
pub use traits::*;
pub use value::*;
pub use value_range::*;
pub use vm::*;
