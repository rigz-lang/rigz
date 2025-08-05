use crate::process::{ProcessManager, ProcessRunner};
use crate::{Modules, Scope, VMOptions};
use rigz_core::{MutableReference, ObjectValue};

#[derive(Debug)]
pub(crate) struct Process {
    pub scope: Scope,
    options: VMOptions,
    modules: Modules,
    pub(crate) timeout: Option<usize>,
    process_manager: MutableReference<ProcessManager>,
}

impl Process {
    pub(crate) fn new(
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
        }
    }

    pub(crate) fn run(&self, args: Vec<ObjectValue>) -> ObjectValue {
        let mut runner = ProcessRunner::new(
            &self.scope,
            args,
            &self.options,
            self.modules.clone(),
            self.process_manager.clone(),
        );
        runner.run()
    }
}
