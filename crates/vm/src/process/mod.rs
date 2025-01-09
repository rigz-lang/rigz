mod runner;

use crate::process::runner::ProcessRunner;
use crate::{ModulesMap, Scope, VMError, VMOptions, Value};
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

type FromChannel = (Sender<Option<Vec<Value>>>, Receiver<Option<Vec<Value>>>);

#[derive(Debug)]
pub struct Process<'vm> {
    pub scope: Scope<'vm>,
    from_vm: FromChannel,
    to_vm: (Sender<Option<Value>>, Receiver<Option<Value>>),
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
            from_vm: unbounded(),
            to_vm: unbounded(),
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
    ) -> (Box<Self>, JoinHandle<Result<(), VMError>>) {
        let p = Self::new(scope, options, modules, timeout);
        // ensure memory address doesn't move if Vec<Process> is re-allocated
        let p = Box::new(p);
        let t = p.start();
        (p, t)
    }

    pub fn close(&self) {
        let _ = &self.from_vm.0.send(None);
        let _ = &self.to_vm.0.send(None);
    }

    fn run(&self, args: Vec<Value>) {
        let mut runner = ProcessRunner::new(&self.scope, args, &self.options, self.modules.clone());
        let v = runner.run();
        self.to_vm.0.send(Some(v)).unwrap()
    }

    pub fn start(&self) -> JoinHandle<Result<(), VMError>> {
        let process: &Process<'static> = unsafe { std::mem::transmute(self) };

        // todo switch to tokio for green threads
        thread::spawn(move || {
            loop {
                let from = &process.from_vm.1;

                match from.recv() {
                    Ok(Some(v)) => process.run(v),
                    Ok(None) | Err(_) => break,
                }
            }

            Ok(())
        })
    }

    pub fn receive(&self, timeout: Option<usize>) -> Value {
        let channel = &self.to_vm.1;
        let timeout = match timeout {
            None => self.timeout,
            Some(t) => Some(t),
        };
        let v = match timeout {
            None => channel.recv().unwrap_or_else(|e| {
                Some(VMError::RuntimeError(format!("Failed to receive value {e:?}")).into())
            }),
            Some(t) => channel
                .recv_timeout(Duration::from_millis(t as u64))
                .unwrap_or_else(|e| {
                    Some(
                        VMError::RuntimeError(format!("`receive` timed out after {t}ms - {e:?}"))
                            .into(),
                    )
                }),
        };
        v.unwrap_or_else(|| {
            VMError::RuntimeError("process exited before message received".to_string()).into()
        })
    }

    pub fn send(&self, args: Vec<Value>) -> Result<(), VMError> {
        let channel = &self.from_vm.0;
        channel
            .send(Some(args))
            .map_err(|e| VMError::RuntimeError(format!("Failed to send - {e:?}")))?;
        Ok(())
    }
}
