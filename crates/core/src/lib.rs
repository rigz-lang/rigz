#[cfg(feature = "derive")]
pub mod derive;

mod args;
mod lifecycle;
mod macros;
mod number;
mod object;
mod operations;
mod primitive;
mod reference;
mod traits;
mod types;
mod vm_values;

pub type IndexMap<K, V> = indexmap::map::IndexMap<K, V>;
pub type IndexMapEntry<'a, K, V> = indexmap::map::Entry<'a, K, V>;

pub use args::RigzArgs;
pub use lifecycle::*;
pub use number::*;
pub use object::*;
pub use operations::*;
pub use primitive::*;
pub use reference::*;
pub use traits::*;
pub use types::*;
pub use vm_values::*;
