use crate::process::{ProcessManager, ProcessRunner};
use crate::{Modules, Scope, VMOptions};
use rigz_core::{Lifecycle, MutableReference, ObjectValue, VMError};
use std::time::Duration;

#[derive(Debug)]
pub struct Process {
    pub scope: Scope,
    options: VMOptions,
    modules: Modules,
    pub(crate) timeout: Option<usize>,
    process_manager: MutableReference<ProcessManager>,
    pub(crate) requests: Vec<Vec<ObjectValue>>,
}

impl Process {
    pub fn new(
        scope: Scope,
        options: VMOptions,
        modules: Modules,
        timeout: Option<usize>,
        process_manager: MutableReference<ProcessManager>,
    ) -> Self {
        Self {
            scope,
            options,
            modules,
            timeout,
            process_manager,
            requests: vec![],
        }
    }

    pub fn receive(&mut self, timeout: Option<usize>) -> ObjectValue {
        if self.requests.is_empty() {
            return VMError::runtime(format!("No requests for process {:?}", self.scope)).into();
        }
        let args = self.requests.remove(0);
        let mut runner = ProcessRunner::new(
            &self.scope,
            args,
            &self.options,
            self.modules.clone(),
            self.process_manager.clone(),
        );
        match timeout {
            None => runner.run(),
            Some(t) => runner.run_within(t),
        }
    }

    pub fn send(&mut self, args: Vec<ObjectValue>) -> Result<(), VMError> {
        self.requests.push(args);
        Ok(())
    }
}
