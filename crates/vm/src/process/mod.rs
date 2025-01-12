use crate::Module;

#[cfg(feature = "threaded")]
mod threaded;

#[cfg(feature = "threaded")]
pub type ModulesMap =
    std::sync::Arc<dashmap::DashMap<&'static str, std::sync::Arc<dyn Module + Send + Sync>>>;

#[cfg(feature = "threaded")]
pub type Reference<T> = std::sync::Arc<T>;

#[cfg(feature = "threaded")]
pub type Process = threaded::Process;

#[cfg(feature = "threaded")]
pub use threaded::SpawnedProcess;

#[cfg(not(feature = "threaded"))]
pub type ModulesMap = std::collections::HashMap<&'static str, Box<dyn Module>>;

#[cfg(not(feature = "threaded"))]
mod single;

#[cfg(not(feature = "threaded"))]
pub type Process = single::Process;

#[cfg(not(feature = "threaded"))]
pub type SpawnedProcess = single::Process;

#[cfg(not(feature = "threaded"))]
pub type Reference<T> = std::rc::Rc<T>;
