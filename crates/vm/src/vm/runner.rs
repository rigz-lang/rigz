use crate::{
    runner_common, CallFrame, CallType, Instruction, ModulesMap, ResolvedModule, Runner, Scope,
    VMOptions, VMState, Variable, VM,
};
use itertools::Itertools;
use log_derive::{logfn, logfn_inputs};
use rigz_core::{
    AsPrimitive, EnumDeclaration, Lifecycle, ObjectValue, ResolveValue, RigzArgs, StackValue,
    VMError,
};
use std::fmt::Display;
use std::ops::Deref;
use std::thread;
use std::time::Duration;

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
            let err = VMError::runtime(format!(
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
                self.load_mut(arg, false)?;
            } else {
                self.load_let(arg, false)?;
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
                            let index = c
                                .iter_mut()
                                .find_position(|l| matches!(l, Lifecycle::Memo(_)));
                            match index {
                                None => {
                                    return Err(VMError::ScopeDoesNotExist(format!(
                                    "Invalid Scope {scope_index}, does not contain @memo lifecycle"
                                )))
                                }
                                Some((index, l)) => {
                                    let Lifecycle::Memo(m) = l else {
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
                let value = self.handle_scope(scope_index).unwrap_or_default();
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

    fn call_dependency(
        &mut self,
        args: RigzArgs,
        dep: usize,
        call_type: CallType,
    ) -> Result<ObjectValue, VMError> {
        match self.dependencies.read() {
            Ok(deps) => match deps.get(dep) {
                None => Err(VMError::runtime(format!("Dependency not found {dep}"))),
                Some(dep) => {
                    let res = match call_type {
                        CallType::Create => {
                            let c = dep.create;
                            c(args)?.into()
                        }
                        CallType::Call(func) => {
                            let c = dep.call;
                            c(func, args)?
                        }
                    };
                    Ok(res)
                }
            },
            Err(e) => Err(VMError::runtime(format!("Failed to get deps {e}"))),
        }
    }

    fn goto(&mut self, scope_id: usize, pc: usize) -> Result<(), VMError> {
        self.sp = scope_id;
        self.frames.current.borrow_mut().pc = pc;
        Ok(())
    }

    fn send(&mut self, args: usize) -> Result<(), VMError> {
        let args = self.resolve_args(args);
        let v = self.process_manager.update(|p| p.send(args))?;
        self.store_value(v.into());
        Ok(())
    }

    fn receive(&mut self, args: usize) -> Result<(), VMError> {
        let args = self.resolve_args(args);
        let res = self.process_manager.update(move |p| p.receive(args))?;
        self.store_value(res.into());
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
        let m = self.modules();
        let pid = self
            .process_manager
            .update_with_ref(move |p, pm| p.spawn(scope, vec![], options, m, timeout, pm))?;
        self.store_value((pid as i64).into());
        Ok(())
    }

    fn find_enum(&mut self, enum_type: usize) -> Result<std::sync::Arc<EnumDeclaration>, VMError> {
        match self
            .enums
            .read()
            .expect("Failed to read enums")
            .get(enum_type)
        {
            None => Err(VMError::runtime(format!("Enum {enum_type} doesn't exist"))),
            Some(v) => Ok(v.clone()),
        }
    }

    fn call(
        &mut self,
        module: ResolvedModule,
        func: String,
        args: usize,
    ) -> Result<ObjectValue, VMError> {
        let args = self.resolve_args(args).into();
        module.call(func, args)
    }
    //
    // fn vm_extension(
    //     &mut self,
    //     module: ResolvedModule,
    //     func: String,
    //     args: usize,
    // ) -> Result<ObjectValue, VMError> {
    //     let args = self.resolve_args(args).into();
    //     module.vm_extension(self, func, args)
    // }

    fn call_loop(&mut self, scope_id: usize) -> Option<VMState> {
        let sp = self.sp;
        let current = self.frames.len();
        if let Err(s) = self.call_frame(scope_id) {
            return Some(s.into());
        }
        let mut result: Option<VMState> = None;
        loop {
            let instruction = match self.next_instruction() {
                None => {
                    // for manual calls to build a loop instruction without a ret
                    if self.frames.current.borrow().scope_id == scope_id {
                        self.frames.current.borrow_mut().pc = 0;
                        self.frames.current.borrow_mut().clear_variables();
                        continue;
                    }
                    break;
                }
                Some(Instruction::Ret)
                    if self.frames.current.borrow().scope_id == scope_id
                        && self.scopes[self.frames.current.borrow().scope_id]
                            .instructions
                            .len()
                            == self.frames.current.borrow().pc =>
                {
                    self.frames.current.borrow_mut().pc = 0;
                    self.frames.current.borrow_mut().clear_variables();
                    continue;
                }
                Some(s) => s,
            };

            match self.process_instruction_scope(instruction) {
                VMState::Running => {}
                VMState::Next => {
                    while self.frames.len() > current + 1
                        && self.frames.current.borrow().parent.is_some()
                    {
                        let frame = match self.frames.pop() {
                            None => {
                                return Some(
                                    VMError::EmptyStack(
                                        "loop processed empty frame - next".to_string(),
                                    )
                                    .into(),
                                )
                            }
                            Some(frame) => frame,
                        };
                        self.frames.current.swap(&frame);
                    }
                    self.frames.current.borrow_mut().pc = 0;
                    self.sp = scope_id;
                }
                VMState::Break => break,
                s => {
                    result = Some(s);
                    break;
                }
            };
        }
        self.sp = sp;
        while self.frames.len() > current {
            let frame = match self.frames.pop() {
                None => {
                    return Some(
                        VMError::EmptyStack("loop processed empty frame - break".to_string())
                            .into(),
                    )
                }
                Some(frame) => frame,
            };
            self.frames.current.swap(&frame);
        }
        result
    }

    fn call_for(&mut self, scope_id: usize) -> Option<VMState> {
        let sp = self.sp;
        let current = self.frames.len();
        let Some(value) = self.pop() else {
            return Some(
                VMError::runtime(format!("No object passed into for loop - {scope_id}")).into(),
            );
        };
        let value = match value.resolve(self).borrow().to_list() {
            Ok(v) => v,
            Err(e) => {
                return Some(
                    VMError::runtime(format!(
                        "Invalid object passed into for loop - {scope_id}, {value:?}"
                    ))
                    .into(),
                )
            }
        };

        let args = match self.scopes.get(scope_id) {
            None => {
                return Some(
                    VMError::runtime(format!("Scope does not exist in for loop - {scope_id}"))
                        .into(),
                )
            }
            Some(v) => v.args.clone(),
        };
        if args.is_empty() {
            return Some(
                VMError::runtime(format!("No args for scope in for loop- {scope_id}")).into(),
            );
        }
        let new_frame = CallFrame::child(scope_id, self.frames.len());
        let old = self.frames.current.replace(new_frame);
        self.frames.push(old);
        self.sp = scope_id;
        let mut result: Option<VMState> = None;
        'outer: for each in value {
            if let ObjectValue::Tuple(tuple) = each {
                for (value, (name, mutable)) in tuple.into_iter().zip(&args) {
                    let res = if *mutable {
                        self.frames.load_mut(name.clone(), value.into(), false)
                    } else {
                        self.frames.load_let(name.clone(), value.into(), false)
                    };
                }
            } else {
                let (name, mutable) = args[0].clone();
                let res = if mutable {
                    self.frames.load_mut(name, each.into(), false)
                } else {
                    self.frames.load_let(name, each.into(), false)
                };

                if let Err(err) = res {
                    result = Some(err.into());
                    break;
                }
            }
            loop {
                let ins = match self.next_instruction() {
                    None => {
                        if self.frames.current.borrow().scope_id == scope_id {
                            self.frames.current.borrow_mut().pc = 0;
                            self.frames.current.borrow_mut().clear_variables();
                            continue;
                        }
                        break 'outer;
                    }
                    Some(Instruction::Ret)
                        if self.frames.current.borrow().scope_id == scope_id
                            && self.scopes[self.frames.current.borrow().scope_id]
                                .instructions
                                .len()
                                == self.frames.current.borrow().pc =>
                    {
                        self.frames.current.borrow_mut().pc = 0;
                        self.frames.current.borrow_mut().clear_variables();
                        break;
                    }
                    Some(i) => i,
                };
                match self.process_instruction_scope(ins) {
                    VMState::Running => {}
                    VMState::Break => {
                        break 'outer;
                    }
                    VMState::Next => {
                        while self.frames.len() > current + 1 {
                            let Some(next) = self.frames.pop() else {
                                return Some(
                                    VMError::runtime("Missing call frame".to_string()).into(),
                                );
                            };
                            self.frames.current.swap(&next);
                        }
                        self.sp = scope_id;
                        self.frames.current.borrow_mut().pc = 0;
                        self.frames.current.borrow_mut().clear_variables();
                        break;
                    }
                    s => {
                        result = Some(s);
                        break 'outer;
                    }
                }
            }
        }
        self.sp = sp;
        while self.frames.len() > current {
            let Some(next) = self.frames.pop() else {
                return Some(VMError::runtime("Missing call frame".to_string()).into());
            };
            self.frames.current.swap(&next);
        }
        result
    }

    fn sleep(&self, duration: Duration) {
        thread::sleep(duration);
    }

    fn exit<V>(&mut self, value: V)
    where
        V: Into<StackValue>,
    {
        let v = value.into();
        let scope_id = self.scopes.len();
        self.scopes.push(Scope {
            args: vec![],
            named: "exit".to_string(),
            instructions: vec![Instruction::Halt],
            lifecycle: None,
            set_self: None,
        });
        let current = self.frames.current.replace(CallFrame::exit(scope_id));
        self.frames.push(current);
        self.stack.push(v);
    }
}
