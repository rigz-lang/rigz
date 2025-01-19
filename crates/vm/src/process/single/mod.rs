use crate::process::ProcessManager;
use crate::{ModulesMap, Scope, VMOptions};
use rigz_core::{Lifecycle, MutableReference, ObjectValue, VMError};

#[derive(Debug)]
pub struct Process {
    pub scope: Scope,
    options: VMOptions,
    modules: ModulesMap,
    timeout: Option<usize>,
    process_manager: MutableReference<ProcessManager>,
}

impl Process {
    pub fn new(
        scope: Scope,
        options: VMOptions,
        modules: ModulesMap,
        timeout: Option<usize>,
        process_manager: MutableReference<ProcessManager>,
    ) -> Self {
        Self {
            scope,
            options,
            modules,
            timeout,
            process_manager,
        }
    }

    pub fn spawn(
        scope: Scope,
        options: VMOptions,
        modules: ModulesMap,
        timeout: Option<usize>,
        process_manager: MutableReference<ProcessManager>,
    ) -> Self {
        Self::new(scope, options, modules, timeout, process_manager)
    }

    pub fn lifecycle(&self) -> Option<&Lifecycle> {
        self.scope.lifecycle.as_ref()
    }

    pub fn close(self) -> Result<(), VMError> {
        Err(VMError::todo(
            "`close` is not implemented for Single Threaded Process".to_string(),
        ))
    }

    pub fn receive(&self, timeout: Option<usize>) -> ObjectValue {
        VMError::todo("`receive` is not implemented for Single Threaded Process".to_string()).into()
    }

    pub fn send(&self, args: Vec<ObjectValue>) -> Result<(), VMError> {
        Err(VMError::todo(
            "`send` is not implemented for Single Threaded Process".to_string(),
        ))
    }
}
