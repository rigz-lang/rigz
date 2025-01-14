use crate::process::{MutableReference, Process};
use crate::{Lifecycle, ModulesMap, Reference, Scope, VMError, VMOptions, Value, VM};
use log::warn;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::time::Duration;

#[derive(Debug)]
pub(crate) enum VMMessage {}

#[derive(Debug)]
pub(crate) enum ProcessMessage {}

#[derive(Debug)]
pub(crate) struct VMMessenger {}

#[derive(Debug)]
pub(crate) struct ProcessManager {
    #[cfg(feature = "threaded")]
    pub(crate) handle: tokio::runtime::Handle,
    #[cfg(feature = "threaded")]
    runtime: Option<tokio::runtime::Runtime>,
    processes: SpawnedProcesses,
    vm_messenger: Option<VMMessenger>,
}

#[cfg(feature = "threaded")]
pub(crate) type SpawnedProcesses =
    Vec<(Reference<Process>, Option<tokio::task::JoinHandle<Value>>)>;

#[cfg(not(feature = "threaded"))]
pub(crate) type SpawnedProcesses = Vec<Reference<Process>>;

#[cfg(feature = "threaded")]
fn run_process(
    handle: &tokio::runtime::Handle,
    id: usize,
    running: &mut (Reference<Process>, Option<tokio::task::JoinHandle<Value>>),
    args: Vec<Value>,
) -> Value {
    let (p, running) = running;
    let p = p.clone();
    let current = running.take();
    let t = match current {
        None => handle.spawn_blocking(move || p.run(args)),
        Some(_) => {
            return VMError::todo(format!(
                "overwriting running tasks is not supported - Process {id}"
            ))
            .into()
        }
    };
    *running = Some(t);
    Value::Number((id as i64).into())
}

impl ProcessManager {
    #[cfg(not(feature = "threaded"))]
    pub(crate) fn new() -> Self {
        Self {
            processes: Vec::new(),
            vm_messenger: None,
        }
    }

    #[cfg(feature = "threaded")]
    pub(crate) fn create() -> Result<Self, VMError> {
        let (handle, runtime) = match tokio::runtime::Handle::try_current() {
            Ok(r) => (r, None),
            Err(_) => match tokio::runtime::Runtime::new() {
                Ok(r) => (r.handle().clone(), Some(r)),
                Err(e) => {
                    return Err(VMError::RuntimeError(format!(
                        "Failed to create tokio runtime {e}"
                    )))
                }
            },
        };

        Ok(Self {
            handle,
            runtime,
            processes: Vec::new(),
            vm_messenger: None,
        })
    }

    pub(crate) fn add(&mut self, processes: SpawnedProcesses) {
        self.processes.extend(processes);
    }

    pub(crate) fn spawn(
        &mut self,
        scope: Scope,
        args: Vec<Value>,
        options: VMOptions,
        modules: ModulesMap,
        timeout: Option<usize>,
        process_manager: MutableReference<ProcessManager>,
    ) -> Result<usize, VMError> {
        let pid = self.processes.len();
        let p: Reference<Process> =
            Process::new(scope, options, modules, timeout, process_manager).into();
        #[cfg(feature = "threaded")]
        {
            let arc = p.clone();
            let t = self.handle.spawn_blocking(move || arc.run(args));
            self.processes.push((p, Some(t)));
        }

        #[cfg(not(feature = "threaded"))]
        {
            self.processes.push(p);
        }
        Ok(pid)
    }

    pub(crate) fn send(&mut self, args: Vec<Rc<RefCell<Value>>>) -> Result<Value, VMError> {
        let mut args = args.into_iter().map(|v| v.borrow().clone());
        // todo message or pid
        let message = args.next().unwrap().to_string();
        let args = Vec::from_iter(args);
        let res: Vec<_> = self
            .processes
            .iter_mut()
            .enumerate()
            .filter(|(_, (p, _))| match p.scope.lifecycle.as_ref() {
                Some(Lifecycle::On(e)) => e.event == message,
                _ => false,
            })
            .map(|(id, running)| run_process(&self.handle, id, running, args.clone()))
            .collect();

        if res.is_empty() {
            return Err(VMError::RuntimeError(format!(
                "No process found matching '{message}'"
            )));
        }

        Ok(res.into())
    }

    pub(crate) fn receive(&mut self, args: Vec<Rc<RefCell<Value>>>) -> Result<Value, VMError> {
        let mut args = args.into_iter().map(|v| v.borrow().clone());
        let v = args.next().unwrap();

        let timeout = match args.next().map(|v| v.to_usize()) {
            Some(Ok(u)) => Some(u),
            Some(Err(e)) => return Err(e),
            None => None,
        };

        let v = match v {
            Value::List(val) => {
                let mut res = Vec::with_capacity(val.len());
                for v in val {
                    let r = match v.to_usize() {
                        Ok(pid) => self.handle_receive(pid, timeout),
                        Err(e) => e.into(),
                    };
                    res.push(r);
                }
                res.into()
            }
            _ => match v.to_usize() {
                Ok(pid) => self.handle_receive(pid, timeout),
                Err(e) => e.into(),
            },
        };
        Ok(v)
    }

    fn handle_receive(&mut self, pid: usize, timeout: Option<usize>) -> Value {
        match self.processes.get_mut(pid) {
            None => VMError::RuntimeError(format!("Process {pid} does not exist")).into(),
            Some((p, t)) => {
                let running = match t {
                    None => {
                        return VMError::RuntimeError(format!("Process {pid} is not running"))
                            .into()
                    }
                    Some(t) => t,
                };
                let timeout = match timeout {
                    None => p.timeout,
                    Some(s) => Some(s),
                };
                let res = self.handle.block_on(async move {
                    match timeout {
                        None => running.await,
                        Some(time) => {
                            match tokio::time::timeout(Duration::from_millis(time as u64), running)
                                .await
                            {
                                Ok(v) => v,
                                Err(e) => Ok(VMError::RuntimeError(format!(
                                    "`receive` timed out after {time}ms - {e}"
                                ))
                                .into()),
                            }
                        }
                    }
                });
                *t = None;

                res.unwrap_or_else(|e| {
                    VMError::RuntimeError(format!("Process {pid} failed: {e}")).into()
                })
            }
        }
    }

    pub(crate) fn close(&mut self, result: Value) -> Value {
        let mut errors: Vec<VMError> = vec![];
        for (id, (_, handle)) in self.processes.drain(..).enumerate() {
            match handle {
                None => {}
                Some(t) => match self.handle.block_on(t) {
                    Ok(v) => {
                        warn!("Orphaned value from Process {id} - {v}")
                    }
                    Err(e) => errors.push(VMError::RuntimeError(format!(
                        "Failed to close process {id} - {e}"
                    ))),
                },
            }
        }

        if errors.is_empty() {
            result
        } else {
            let len = errors.len() - 1;
            let messages =
                errors
                    .iter()
                    .enumerate()
                    .fold(String::new(), |mut res, (index, next)| {
                        res.push_str(next.to_string().as_str());
                        if index != len {
                            res.push_str(", ");
                        }
                        res
                    });
            VMError::RuntimeError(format!("Process Failures: {messages}")).into()
        }
    }

    // todo return channel
    pub(crate) fn create_on_processes(vm: &VM) -> SpawnedProcesses {
        let scopes = vm
            .scopes
            .iter()
            .filter(|s| matches!(s.lifecycle, Some(Lifecycle::On(_))))
            .map(|s| {
                Process::new(
                    s.clone(),
                    vm.options,
                    vm.modules.clone(),
                    None,
                    vm.process_manager.clone(),
                )
                .into()
            });

        #[cfg(feature = "threaded")]
        {
            scopes.map(|p| (p, None)).collect()
        }

        #[cfg(not(feature = "threaded"))]
        {
            scopes.map(|p| (p, None)).collect()
        }
    }
}
