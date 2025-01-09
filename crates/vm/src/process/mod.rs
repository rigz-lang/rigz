use crate::Module;

#[cfg(feature = "threaded")]
mod threaded;

#[cfg(feature = "threaded")]
pub type ModulesMap<'vm> =
    std::sync::Arc<dashmap::DashMap<&'static str, std::sync::Arc<dyn Module<'vm> + Send + Sync>>>;

#[cfg(feature = "threaded")]
pub type Process<'vm> = threaded::Process<'vm>;

#[cfg(feature = "threaded")]
pub use threaded::SpawnedProcess;

#[cfg(not(feature = "threaded"))]
pub type ModulesMap<'vm> = std::collections::HashMap<&'static str, Box<dyn Module<'vm>>>;

#[cfg(not(feature = "threaded"))]
mod single;

#[cfg(not(feature = "threaded"))]
pub type Process<'vm> = single::Process<'vm>;

#[cfg(not(feature = "threaded"))]
pub type SpawnedProcess<'vm> = single::Process<'vm>;
