use crate::{
    runner_common, BroadcastArgs, CallFrame, Lifecycle, Number, Process, ResolveValue,
    ResolvedModule, Runner, Scope, StackValue, VMError, VMOptions, Value, Variable, VM,
};
use itertools::Itertools;
use log_derive::{logfn, logfn_inputs};
use std::fmt::Display;
use std::ops::Deref;

macro_rules! broadcast {
    ($args:expr, $ex: expr) => {
        let args: Vec<Value> = $args.collect();
        $ex.map(|(id, p)| (id, p.send(args.clone())))
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
            $ex.filter(|(_, p)| match p.lifecycle() {
                    Some(Lifecycle::On(e)) => e.event == message,
                    _ => false,
                })
        }
    };
}

#[allow(unused_variables)]
impl<'vm> Runner<'vm> for VM<'vm> {
    runner_common!();

    fn update_scope<F>(&mut self, index: usize, mut update: F) -> Result<(), VMError>
    where
        F: FnMut(&mut Scope<'vm>) -> Result<(), VMError>,
    {
        match self.scopes.get_mut(index) {
            None => Err(VMError::ScopeDoesNotExist(format!(
                "Scope {index} does not exist"
            ))),
            Some(s) => update(s),
        }
    }

    #[inline]
    fn call_frame(&mut self, scope_index: usize) -> Result<(), VMError> {
        if self.scopes.len() <= scope_index {
            return Err(VMError::ScopeDoesNotExist(format!(
                "{} does not exist",
                scope_index
            )));
        }

        if self.frames.len() >= self.options.max_depth {
            let err = VMError::RuntimeError(format!(
                "Stack overflow: exceeded {}",
                self.options.max_depth
            ));
            return Err(err);
        }

        let current = self
            .frames
            .current
            .replace(CallFrame::child(scope_index, self.frames.len()));
        self.frames.push(current);
        self.sp = scope_index;

        if let Some(mutable) = self.scopes[scope_index].set_self {
            self.set_this(mutable)?;
        }

        for (arg, mutable) in self.scopes[scope_index].args.clone() {
            if mutable {
                self.load_mut(arg)?;
            } else {
                self.load_let(arg)?;
            }
        }
        Ok(())
    }

    fn call_frame_memo(&mut self, scope_index: usize) -> Result<(), VMError> {
        let args = self.scopes[scope_index].args.len();
        let call_args = if self.scopes[scope_index].set_self.is_some() {
            let mut ca = Vec::with_capacity(args + 1);
            ca.push(self.next_resolved_value("call frame_memo"));
            ca.extend(self.resolve_args(args));
            ca
        } else {
            self.resolve_args(args)
        };
        let value = match self.scopes.get_mut(scope_index) {
            None => {
                return Err(VMError::ScopeDoesNotExist(format!(
                    "Invalid Scope {scope_index}"
                )))
            }
            Some(s) => match &mut s.lifecycle {
                None => {
                    return Err(VMError::ScopeDoesNotExist(format!(
                        "Invalid Scope {scope_index}, does not contain @memo lifecycle"
                    )))
                }
                Some(l) => {
                    let call_args: Vec<_> = call_args.iter().map(|v| v.borrow().clone()).collect();
                    let memo = match l {
                        Lifecycle::Memo(m) => m,
                        Lifecycle::Composite(c) => {
                            let index = c.iter().find_position(|l| matches!(l, Lifecycle::Memo(_)));
                            match index {
                                None => {
                                    return Err(VMError::ScopeDoesNotExist(format!(
                                    "Invalid Scope {scope_index}, does not contain @memo lifecycle"
                                )))
                                }
                                Some((index, _)) => {
                                    let Lifecycle::Memo(m) = c.get_mut(index).unwrap() else {
                                        unreachable!()
                                    };
                                    m
                                }
                            }
                        }
                        _ => {
                            return Err(VMError::ScopeDoesNotExist(format!(
                                "Invalid Scope {scope_index}, does not contain @memo lifecycle"
                            )))
                        }
                    };

                    memo.results.get(&call_args).cloned()
                }
            },
        };
        let value = match value {
            None => {
                call_args
                    .iter()
                    .rev()
                    .for_each(|v| self.stack.push(v.clone().into()));
                let value = self.handle_scope(scope_index);
                let s = self.scopes.get_mut(scope_index).unwrap();
                match &mut s.lifecycle {
                    None => unreachable!(),
                    Some(l) => {
                        let memo = match l {
                            Lifecycle::Memo(m) => m,
                            Lifecycle::Composite(c) => {
                                let (index, _) = c
                                    .iter()
                                    .find_position(|l| matches!(l, Lifecycle::Memo(_)))
                                    .unwrap();
                                let Lifecycle::Memo(m) = c.get_mut(index).unwrap() else {
                                    unreachable!()
                                };
                                m
                            }
                            _ => unreachable!(),
                        };

                        let call_args = call_args.into_iter().map(|v| v.borrow().clone()).collect();
                        memo.results.insert(call_args, value.borrow().clone());
                        value
                    }
                }
            }
            Some(s) => s.into(),
        };
        self.store_value(value.into());
        Ok(())
    }

    fn goto(&mut self, scope_id: usize, pc: usize) -> Result<(), VMError> {
        self.sp = scope_id;
        self.frames.current.borrow_mut().pc = pc;
        Ok(())
    }

    fn send(&mut self, args: usize) -> Result<(), VMError> {
        let mut args = self
            .resolve_args(args)
            .into_iter()
            .map(|v| v.borrow().clone());
        let message = args.next().unwrap().to_string();
        let process = self
            .processes
            .iter()
            .find_position(|p| match p.lifecycle() {
                Some(Lifecycle::On(e)) => e.event == message,
                _ => false,
            });

        let v = match process {
            None => {
                return Err(VMError::RuntimeError(format!(
                    "No process found matching '{message}'"
                )))
            }
            Some((id, p)) => match p.send(Vec::from_iter(args)) {
                Ok(_) => Value::Number((id as i64).into()),
                Err(e) => e.into(),
            },
        };
        self.store_value(v.into());
        Ok(())
    }

    fn receive(&mut self, args: usize) -> Result<(), VMError> {
        let mut args = self
            .resolve_args(args)
            .into_iter()
            .map(|v| v.borrow().clone());
        let v = args.next().unwrap();

        let timeout = match args.next().map(|v| v.to_usize()) {
            Some(Ok(u)) => Some(u),
            Some(Err(e)) => return Err(e),
            None => None,
        };

        let res = match v {
            Value::List(val) => {
                let mut res = Vec::with_capacity(val.len());
                for v in val {
                    let pid = v.to_usize()?;
                    let r = match self.processes.get(pid) {
                        None => {
                            VMError::RuntimeError(format!("Process {pid} does not exist")).into()
                        }
                        Some(p) => p.receive(timeout),
                    };
                    res.push(r);
                }
                res.into()
            }
            _ => {
                let pid = v.to_usize()?;
                match self.processes.get(pid) {
                    None => VMError::RuntimeError(format!("Process {pid} does not exist")).into(),
                    Some(p) => p.receive(timeout),
                }
            }
        };

        self.store_value(res.into());
        Ok(())
    }

    fn broadcast(&mut self, args: BroadcastArgs) -> Result<(), VMError> {
        let (all, args) = match args {
            BroadcastArgs::Args(a) => (false, a),
            BroadcastArgs::All(a) => (true, a),
        };

        let mut args = self
            .resolve_args(args)
            .into_iter()
            .map(|v| v.borrow().clone());

        let values = if all {
            broadcast! { args, self.processes.iter().enumerate() }
        } else {
            broadcast! { message: args, self
            .processes
            .iter()
            .enumerate() }
        };
        self.store_value(values.into());
        Ok(())
    }

    fn spawn(&mut self, scope_id: usize, timeout: Option<usize>) -> Result<(), VMError> {
        let scope = match self.scopes.get(scope_id) {
            None => {
                return Err(VMError::ScopeDoesNotExist(format!(
                    "Scope does not exist - {scope_id}"
                )))
            }
            Some(s) => s.clone(),
        };
        let options = self.options;
        let m = self.modules.clone();
        let pid = self.processes.len();
        let p = Process::spawn(scope, options, m, timeout);
        p.send(vec![])?;
        self.processes.push(p);
        self.store_value(Number::Int(pid as i64).into());
        Ok(())
    }

    fn call(
        &mut self,
        module: ResolvedModule<'vm>,
        func: &'vm str,
        args: usize,
    ) -> Result<Value, VMError> {
        let args = self.resolve_args(args).into();
        module.call(func, args)
    }

    fn call_extension(
        &mut self,
        module: ResolvedModule<'vm>,
        func: &'vm str,
        args: usize,
    ) -> Result<Value, VMError> {
        let this = self.next_resolved_value("call_extension");
        let args = self.resolve_args(args).into();
        module.call_extension(this, func, args)
    }

    fn call_mutable_extension(
        &mut self,
        module: ResolvedModule<'vm>,
        func: &'vm str,
        args: usize,
    ) -> Result<Option<Value>, VMError> {
        let this = self.next_resolved_value("call_extension");
        let args = self.resolve_args(args).into();
        module.call_mutable_extension(this, func, args)
    }

    fn vm_extension(
        &mut self,
        module: ResolvedModule<'vm>,
        func: &'vm str,
        args: usize,
    ) -> Result<Value, VMError> {
        let args = self.resolve_args(args).into();
        module.vm_extension(self, func, args)
    }
}
