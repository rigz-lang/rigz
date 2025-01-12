use crate::process::SpawnedProcess;
use crate::VMError;

#[derive(Debug)]
pub enum VMMessage {}

#[derive(Debug)]
pub enum ProcessMessage {}

#[derive(Debug)]
pub struct VMMessenger {}

#[derive(Debug)]
pub struct ProcessManager {
    #[cfg(feature = "threaded")]
    handle: tokio::runtime::Handle,
    processes: Vec<SpawnedProcess>,
    vm_messenger: Option<VMMessenger>,
}

impl ProcessManager {
    #[cfg(not(feature = "threaded"))]
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            vm_messenger: None,
        }
    }

    #[cfg(feature = "threaded")]
    pub fn create() -> Result<Self, VMError> {
        let handle = match tokio::runtime::Handle::try_current() {
            Ok(r) => r,
            Err(_) => match tokio::runtime::Runtime::new() {
                Ok(r) => r.handle().clone(),
                Err(e) => {
                    return Err(VMError::RuntimeError(format!(
                        "Failed to create tokio runtime {e}"
                    )))
                }
            },
        };

        Ok(Self {
            handle,
            processes: Vec::new(),
            vm_messenger: None,
        })
    }
}
