use crate::process::ModulesMap;
use crate::{Lifecycle, Scope, VMError, VMOptions, Value};
use std::thread::JoinHandle;

#[derive(Debug)]
pub struct Process<'vm> {
    pub scope: Scope<'vm>,
    options: VMOptions,
    modules: ModulesMap<'vm>,
    timeout: Option<usize>,
}

impl<'vm> Process<'vm> {
    pub fn new(
        scope: Scope<'vm>,
        options: VMOptions,
        modules: ModulesMap<'vm>,
        timeout: Option<usize>,
    ) -> Self {
        Self {
            scope,
            options,
            modules,
            timeout,
        }
    }

    pub fn spawn(
        scope: Scope<'vm>,
        options: VMOptions,
        modules: ModulesMap<'vm>,
        timeout: Option<usize>,
    ) -> Self {
        Self::new(scope, options, modules, timeout)
    }

    pub fn lifecycle(&self) -> Option<&Lifecycle> {
        self.scope.lifecycle.as_ref()
    }

    pub fn close(self) -> Result<(), VMError> {
        Err(VMError::todo(
            "`close` is not implemented for Single Threaded Process".to_string(),
        ))
    }

    pub fn receive(&self, timeout: Option<usize>) -> Value {
        VMError::todo("`receive` is not implemented for Single Threaded Process".to_string()).into()
    }

    pub fn send(&self, args: Vec<Value>) -> Result<(), VMError> {
        Err(VMError::todo(
            "`send` is not implemented for Single Threaded Process".to_string(),
        ))
    }
}
