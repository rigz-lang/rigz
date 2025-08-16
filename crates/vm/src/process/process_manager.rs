use crate::process::Process;
use crate::{Modules, Scope, VMOptions, VM};
use log::warn;
use rigz_core::{AsPrimitive, Lifecycle, MutableReference, ObjectValue, Reference, VMError};
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::time::Duration;

#[derive(Debug)]
pub(crate) struct ProcessManager {
    processes: SpawnedProcesses,
}

#[cfg(feature = "threaded")]
pub(crate) type SpawnedProcesses = Vec<(
    Reference<Process>,
    Option<tokio::task::JoinHandle<ObjectValue>>,
)>;

#[cfg(not(feature = "threaded"))]
pub(crate) type SpawnedProcesses = Vec<MutableReference<Process>>;

#[cfg(feature = "threaded")]
fn run_process(
    handle: &tokio::runtime::Handle,
    id: usize,
    running: &mut (
        Reference<Process>,
        Option<tokio::task::JoinHandle<ObjectValue>>,
    ),
    args: Vec<ObjectValue>,
) -> ObjectValue {
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
    (id as i64).into()
}

#[cfg(feature = "threaded")]
static TOKIO: std::sync::LazyLock<
    Result<(tokio::runtime::Handle, Option<tokio::runtime::Runtime>), VMError>,
> = std::sync::LazyLock::new(|| {
    let (handle, runtime) = match tokio::runtime::Handle::try_current() {
        Ok(r) => (r, None),
        Err(_) => match tokio::runtime::Runtime::new() {
            Ok(r) => (r.handle().clone(), Some(r)),
            Err(e) => {
                return Err(VMError::runtime(format!(
                    "Failed to create tokio runtime {e}"
                )))
            }
        },
    };
    Ok((handle, runtime))
});

#[cfg(feature = "threaded")]
fn tokio_handle() -> Result<tokio::runtime::Handle, VMError> {
    match TOKIO.as_ref() {
        Ok((h, _)) => Ok(h.clone()),
        Err(e) => Err(e.clone()),
    }
}

impl ProcessManager {
    #[cfg(not(feature = "threaded"))]
    pub(crate) fn new() -> Self {
        Self {
            processes: Vec::new(),
        }
    }

    #[cfg(feature = "threaded")]
    pub(crate) fn sleep(&self, duration: Duration) {
        if let Ok(handle) = tokio_handle() {
            handle.block_on(tokio::time::sleep(duration));
        }
    }

    #[cfg(feature = "threaded")]
    pub(crate) fn create() -> Result<Self, VMError> {
        Ok(Self {
            processes: Vec::new(),
        })
    }

    pub(crate) fn add(&mut self, processes: SpawnedProcesses) {
        self.processes.extend(processes);
    }

    pub(crate) fn spawn(
        &mut self,
        scope: Scope,
        args: Vec<ObjectValue>,
        options: VMOptions,
        modules: Modules,
        timeout: Option<usize>,
        process_manager: MutableReference<ProcessManager>,
    ) -> Result<usize, VMError> {
        let pid = self.processes.len();
        let p = Process::new(scope, options, modules, timeout, process_manager);
        #[cfg(feature = "threaded")]
        {
            let p: Reference<Process> = p.into();
            let arc = p.clone();
            let handle = tokio_handle()?;
            let t = handle.spawn_blocking(move || arc.run(args));
            self.processes.push((p, Some(t)));
        }

        #[cfg(not(feature = "threaded"))]
        {
            let mut p = p;
            p.requests.push(args);
            self.processes.push(p.into());
        }
        Ok(pid)
    }

    #[cfg(feature = "threaded")]
    pub(crate) fn send(
        &mut self,
        args: Vec<Rc<RefCell<ObjectValue>>>,
    ) -> Result<ObjectValue, VMError> {
        let mut args = args.into_iter().map(|v| v.borrow().deep_clone());
        // todo message or pid
        let message = args.next().unwrap().to_string();
        let args = Vec::from_iter(args);
        let handle = tokio_handle()?;
        let res: Vec<_> = self
            .processes
            .iter_mut()
            .enumerate()
            .filter(|(_, (p, _))| match p.scope.lifecycle.as_ref() {
                Some(Lifecycle::On(e)) => e.event == message,
                _ => false,
            })
            .map(|(id, running)| run_process(&handle, id, running, args.clone()))
            .collect();

        if res.is_empty() {
            return Err(VMError::runtime(format!(
                "No process found matching '{message}'"
            )));
        }

        Ok(res.into())
    }

    #[cfg(not(feature = "threaded"))]
    pub(crate) fn send(
        &mut self,
        args: Vec<Rc<RefCell<ObjectValue>>>,
    ) -> Result<ObjectValue, VMError> {
        let mut args = args.into_iter().map(|v| v.borrow().clone());
        // todo message or pid
        let message = args.next().unwrap().to_string();
        let args = Vec::from_iter(args);
        let res: Vec<Result<ObjectValue, VMError>> = self
            .processes
            .iter_mut()
            .enumerate()
            .filter(|(_, p)| {
                p.apply(|p| match p.scope.lifecycle.as_ref() {
                    Some(Lifecycle::On(e)) => e.event == message,
                    _ => false,
                })
            })
            .map(|(id, running)| {
                running
                    .apply_mut(|r| r.send(args.clone()))
                    .map(|_| (id as i64).into())
            })
            .collect();

        if res.is_empty() {
            return Err(VMError::runtime(format!(
                "No process found matching '{message}'"
            )));
        }

        Ok(res.into())
    }

    pub(crate) fn receive(
        &mut self,
        args: Vec<Rc<RefCell<ObjectValue>>>,
    ) -> Result<ObjectValue, VMError> {
        let mut args = args.into_iter().map(|v| v.borrow().clone());
        let v = args.next().unwrap();

        let timeout = match args.next().map(|v| v.to_usize()) {
            Some(Ok(u)) => Some(u),
            Some(Err(e)) => return Err(e),
            None => None,
        };

        let v = match v {
            ObjectValue::List(val) => {
                let mut res = Vec::with_capacity(val.len());
                for v in val {
                    let r = match v.borrow().to_usize() {
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

    #[cfg(feature = "threaded")]
    fn handle_receive(&mut self, pid: usize, timeout: Option<usize>) -> ObjectValue {
        match self.processes.get_mut(pid) {
            None => VMError::runtime(format!("Process {pid} does not exist")).into(),
            Some((p, t)) => {
                let running = match t {
                    None => {
                        return VMError::runtime(format!("Process {pid} is not running")).into()
                    }
                    Some(t) => t,
                };
                let timeout = match timeout {
                    None => p.timeout,
                    Some(s) => Some(s),
                };
                let handle = match tokio_handle() {
                    Ok(h) => h,
                    Err(e) => return e.into(),
                };
                let res = handle.block_on(async move {
                    match timeout {
                        None => running.await,
                        Some(time) => {
                            match tokio::time::timeout(Duration::from_millis(time as u64), running)
                                .await
                            {
                                Ok(v) => v,
                                Err(e) => Ok(VMError::runtime(format!(
                                    "`receive` timed out after {time}ms - {e}"
                                ))
                                .into()),
                            }
                        }
                    }
                });
                *t = None;

                res.unwrap_or_else(|e| {
                    VMError::runtime(format!("Process {pid} failed: {e}")).into()
                })
            }
        }
    }

    #[cfg(not(feature = "threaded"))]
    fn handle_receive(&mut self, pid: usize, timeout: Option<usize>) -> ObjectValue {
        match self.processes.get(pid) {
            None => VMError::runtime(format!("Process {pid} does not exist")).into(),
            Some(p) => p.apply_mut(|p| {
                let timeout = match timeout {
                    None => p.timeout,
                    Some(s) => Some(s),
                };
                p.receive(timeout)
            }),
        }
    }

    #[cfg(feature = "threaded")]
    pub(crate) fn close(&mut self, result: ObjectValue) -> ObjectValue {
        if self.processes.is_empty() {
            return result
        }
        let mut errors: Vec<VMError> = vec![];
        let tokio = match tokio_handle() {
            Ok(h) => h,
            Err(e) => return e.into(),
        };
        for (id, (_, handle)) in self.processes.drain(..).enumerate() {
            match handle {
                None => {}
                Some(t) => match tokio.block_on(t) {
                    Ok(v) => {
                        warn!("Orphaned value from Process {id} - {v}")
                    }
                    Err(e) => errors.push(VMError::runtime(format!(
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
            VMError::runtime(format!("Process Failures: {messages}")).into()
        }
    }

    #[cfg(not(feature = "threaded"))]
    pub(crate) fn close(&mut self, result: ObjectValue) -> ObjectValue {
        let mut errors: Vec<VMError> = vec![];
        for (id, p) in self.processes.drain(..).enumerate() {
            p.apply_mut(|p| {
                let runs = p.requests.len();
                for _ in 0..runs {
                    let v = p.receive(None);
                    if v.is_error() {
                        errors.push(VMError::runtime(format!(
                            "Failed to close process {id} - {v}"
                        )));
                    } else {
                        warn!("Orphaned value from Process {id} - {v}");
                    }
                }
            });
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
            VMError::runtime(format!("Process Failures: {messages}")).into()
        }
    }

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
            scopes.collect()
        }
    }
}
