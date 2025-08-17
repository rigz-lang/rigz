mod builder;
mod call_frame;
mod instructions;
mod macros;
mod scope;
mod vm;

mod process;
mod stack;

pub use builder::{RigzBuilder, VMBuilder};
pub use call_frame::{CallFrame, Variable};
pub use instructions::*;
pub use scope::Scope;
pub use stack::VMStack;
pub use vm::*;


#[cfg(feature = "std_capture")]
pub trait StdOutCapture: Send + Sync {
    fn applied(&self, value: String);
}

#[cfg(feature = "std_capture")]
pub struct StdCapture {
    pub out: std::sync::RwLock<Option<std::sync::Arc<dyn StdOutCapture>>>,
    pub err: std::sync::RwLock<Option<std::sync::Arc<dyn StdOutCapture>>>,
}

#[cfg(feature = "std_capture")]
pub static CAPTURE: StdCapture = StdCapture {
    out: std::sync::RwLock::new(None),
    err: std::sync::RwLock::new(None),
};