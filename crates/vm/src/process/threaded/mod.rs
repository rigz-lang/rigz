mod runner;

use crate::process::ModulesMap;
use crate::{Scope, VMOptions, Value};
use runner::ProcessRunner;

#[derive(Clone, Debug)]
pub struct Process {
    pub scope: Scope,
    options: VMOptions,
    modules: ModulesMap,
    pub timeout: Option<usize>,
}

impl Process {
    pub fn new(
        scope: Scope,
        options: VMOptions,
        modules: ModulesMap,
        timeout: Option<usize>,
    ) -> Self {
        Self {
            scope,
            options,
            modules,
            timeout,
        }
    }

    pub fn run(&self, args: Vec<Value>) -> Value {
        let mut runner = ProcessRunner::new(&self.scope, args, &self.options, self.modules.clone());
        runner.run()
    }
}
