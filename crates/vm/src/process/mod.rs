#[cfg(feature = "threaded")]
mod threaded;

#[cfg(feature = "threaded")]
pub(crate) type Process = threaded::Process;

mod process_manager;
mod runner;

#[cfg(not(feature = "threaded"))]
mod single;

pub(crate) use process_manager::ProcessManager;
pub(crate) use runner::ProcessRunner;

#[cfg(not(feature = "threaded"))]
pub type Process = single::Process;
