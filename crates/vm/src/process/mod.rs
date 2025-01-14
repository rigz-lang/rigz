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

#[cfg(not(feature = "threaded"))]
pub type ModulesMap = std::collections::HashMap<&'static str, std::rc::Rc<dyn Module>>;

mod process_manager;
#[cfg(not(feature = "threaded"))]
mod single;

pub(crate) use process_manager::ProcessManager;

#[cfg(not(feature = "threaded"))]
pub type Process = single::Process;

#[cfg(not(feature = "threaded"))]
pub type Reference<T> = std::rc::Rc<T>;
