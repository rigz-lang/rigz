use crate::process::ModulesMap;
use crate::{
    runner_common, CallFrame, Lifecycle, Number, Process, Reference, ResolveValue, ResolvedModule,
    Runner, Scope, StackValue, VMError, VMOptions, Value, Variable, VM,
};
use itertools::Itertools;
use log_derive::{logfn, logfn_inputs};
use std::fmt::Display;
use std::ops::Deref;

#[allow(unused_variables)]
impl Runner for VM {
    runner_common!();

    fn update_scope<F>(&mut self, index: usize, mut update: F) -> Result<(), VMError>
    where
        F: FnMut(&mut Scope) -> Result<(), VMError>,
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
        let args = self.resolve_args(args);
        let v = self.process_manager.send(args)?;
        self.store_value(v.into());
        Ok(())
    }

    fn receive(&mut self, args: usize) -> Result<(), VMError> {
        let args = self.resolve_args(args);
        let res = self.process_manager.receive(args);
        self.store_value(res.into());
        Ok(())
    }

    fn broadcast(&mut self, args: usize) -> Result<(), VMError> {
        let args = self.resolve_args(args);
        let values = self.process_manager.broadcast(args);
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
        let options = self.options.clone();
        let m = self.modules();
        let pid = self
            .process_manager
            .spawn(scope, vec![], options, m, timeout)?;
        self.store_value(Number::Int(pid as i64).into());
        Ok(())
    }

    fn call(
        &mut self,
        module: ResolvedModule,
        func: Reference<String>,
        args: usize,
    ) -> Result<Value, VMError> {
        let args = self.resolve_args(args).into();
        module.call(func, args)
    }

    fn vm_extension(
        &mut self,
        module: ResolvedModule,
        func: Reference<String>,
        args: usize,
    ) -> Result<Value, VMError> {
        let args = self.resolve_args(args).into();
        module.vm_extension(self, func, args)
    }
}
