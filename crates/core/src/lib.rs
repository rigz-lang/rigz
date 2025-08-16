#[cfg(feature = "derive")]
pub mod derive;

mod args;
mod enum_value;
mod lifecycle;
mod macros;
mod number;
mod object;
mod operations;
mod primitive;
mod reference;
mod rigz_object;
mod traits;
mod types;
mod vm_values;

pub type IndexMap<K, V> = indexmap::map::IndexMap<K, V, FxBuildHasher>;
pub type IndexSet<V> = indexmap::set::IndexSet<V, FxBuildHasher>;
pub type IndexMapEntry<'a, K, V> = indexmap::map::Entry<'a, K, V>;

pub use args::RigzArgs;
pub use enum_value::*;
use fxhash::FxBuildHasher;
pub use lifecycle::*;
pub use number::*;
pub use object::*;
pub use operations::*;
pub use primitive::*;
pub use reference::*;
pub use rigz_object::RigzObject;
pub use traits::*;
pub use types::*;
pub use vm_values::*;
