use crate::{
    Lifecycle, ModulesMap, Process, Reference, Runner, Scope, VMError, VMOptions, Value, VM,
};
use log::warn;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::time::Duration;
use tokio::task::JoinHandle;

macro_rules! broadcast {
    ($args:expr, $ex: expr) => {
        let args: Vec<Value> = $args.collect();
        $ex.map(|(id, (p, _))| (id, p.send(args.clone())))
            .map(|(id, r)| match r {
                Ok(_) => Value::Number((id as i64).into()),
                Err(e) => e.into(),
            })
            .collect::<Vec<_>>()
    };
    (message: $args:expr, $ex: expr) => {
        let message = $args.next().unwrap().to_string();
        broadcast! {
            $args,
            $ex.filter(|(_, (p, _))| match p.scope.lifecycle.as_ref() {
                    Some(Lifecycle::On(e)) => e.event == message,
                    _ => false,
                })
        }
    };
}

#[derive(Debug)]
pub(crate) enum VMMessage {}

#[derive(Debug)]
pub(crate) enum ProcessMessage {}

#[derive(Debug)]
pub(crate) struct VMMessenger {}

#[derive(Debug)]
pub(crate) struct ProcessManager {
    #[cfg(feature = "threaded")]
    handle: tokio::runtime::Handle,
    #[cfg(feature = "threaded")]
    runtime: Option<tokio::runtime::Runtime>,
    processes: Vec<(Reference<Process>, Option<JoinHandle<Value>>)>,
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

    pub(crate) fn add(&mut self, processes: Vec<(Reference<Process>, Option<JoinHandle<Value>>)>) {
        self.processes.extend(processes);
    }

    pub(crate) fn broadcast(&mut self, args: Vec<Rc<RefCell<Value>>>) -> Vec<Value> {
        let args: Vec<_> = args.into_iter().map(|a| a.borrow().clone()).collect();

        self.processes
            .iter_mut()
            .enumerate()
            .filter(|(_, (p, _))| matches!(p.scope.lifecycle.as_ref(), Some(Lifecycle::On(_))))
            .map(|(id, (p, running))| {
                let p = p.clone();
                let current = std::mem::replace(running, None);
                let args = args.clone();
                let t =
                    match current {
                        None => self.handle.spawn_blocking(move || p.run(args)),
                        Some(_) => return VMError::todo(format!(
                            "Broadcast does not support overwriting running tasks - Process {id}"
                        ))
                        .into(),
                    };
                *running = Some(t);
                Value::Number((id as i64).into())
            })
            .collect()
    }

    pub(crate) fn spawn(
        &mut self,
        scope: Scope,
        args: Vec<Value>,
        options: VMOptions,
        modules: ModulesMap,
        timeout: Option<usize>,
    ) -> Result<usize, VMError> {
        let pid = self.processes.len();
        let p: Reference<Process> = Process::new(scope, options, modules, timeout).into();
        let arc = p.clone();
        let t = self.handle.spawn_blocking(move || arc.run(args));
        self.processes.push((p, Some(t)));
        Ok(pid)
    }

    pub(crate) fn send(&mut self, args: Vec<Rc<RefCell<Value>>>) -> Result<Value, VMError> {
        let mut args = args.into_iter().map(|v| v.borrow().clone());
        // todo message or pid
        let message = args.next().unwrap().to_string();
        let args = Vec::from_iter(args);
        let res: Vec<_> =
            self.processes
                .iter_mut()
                .enumerate()
                .filter(|(_, (p, _))| match p.scope.lifecycle.as_ref() {
                    Some(Lifecycle::On(e)) => e.event == message,
                    _ => false,
                })
                .map(|(id, (p, running))| {
                    let p = p.clone();
                    let current = std::mem::replace(running, None);
                    let args = args.clone();
                    let t =
                        match current {
                            None => self.handle.spawn_blocking(move || p.run(args)),
                            Some(_) => return VMError::todo(format!(
                                "Send does not support overwriting running tasks - Process {id}"
                            ))
                            .into(),
                        };
                    *running = Some(t);
                    Value::Number((id as i64).into())
                })
                .collect();

        match res.len() {
            0 => Err(VMError::RuntimeError(format!(
                "No process found matching '{message}'"
            ))),
            1 => {
                let mut v = res.into_iter();
                Ok(v.next().unwrap().into())
            }
            _ => Ok(res.into()),
        }
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
                        Ok(pid) => self
                            .handle_receive(pid, timeout)
                            .unwrap_or_else(|e| e.into()),
                        Err(e) => e.into(),
                    };
                    res.push(r);
                }
                res.into()
            }
            _ => match v.to_usize() {
                Ok(pid) => self
                    .handle_receive(pid, timeout)
                    .unwrap_or_else(|e| e.into()),
                Err(e) => e.into(),
            },
        };
        Ok(v)
    }

    fn handle_receive(&mut self, pid: usize, timeout: Option<usize>) -> Result<Value, VMError> {
        match self.processes.get_mut(pid) {
            None => Err(VMError::RuntimeError(format!("Process {pid} does not exist")).into()),
            Some((p, t)) => {
                let running = match t {
                    None => {
                        return Err(VMError::RuntimeError(format!(
                            "Process {pid} is not running"
                        )))
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

                match res {
                    Ok(v) => Ok(v),
                    Err(e) => Err(VMError::RuntimeError(format!("Process {pid} failed: {e}"))),
                }
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
    pub(crate) fn create_on_processes(
        vm: &VM,
    ) -> Vec<(Reference<Process>, Option<JoinHandle<Value>>)> {
        // todo should this be an extend?
        vm.scopes
            .iter()
            .filter(|s| matches!(s.lifecycle, Some(Lifecycle::On(_))))
            .map(|s| {
                (
                    Process::new(s.clone(), vm.options, vm.modules.clone(), None).into(),
                    None,
                )
            })
            .collect()
    }
}
