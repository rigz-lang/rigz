use crate::call_frame::{CallFrame, Frames};
use crate::{
    runner_common, BroadcastArgs, Instruction, Module, ModulesMap, ResolveValue, ResolvedModule,
    Runner, Scope, StackValue, VMError, VMOptions, VMStack, VMState, Value, Variable,
};
use log_derive::{logfn, logfn_inputs};
use std::cell::RefCell;
use std::fmt::Display;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

pub(crate) struct ProcessRunner<'s, 'vm> {
    scope: &'s Scope<'vm>,
    frames: Frames<'vm>,
    stack: VMStack,
    options: &'s VMOptions,
    modules: ModulesMap<'vm>,
}

#[allow(unused_variables)]
impl ResolveValue for ProcessRunner<'_, '_> {
    fn location(&self) -> &'static str {
        "Process"
    }

    fn handle_scope(&mut self, scope: usize) -> Rc<RefCell<Value>> {
        VMError::todo("Process does not implement `handle_scope`").into()
    }

    fn get_constant(&self, constant_id: usize) -> Rc<RefCell<Value>> {
        VMError::todo("Process does not implement `get_constant`").into()
    }
}

impl<'s, 'vm> ProcessRunner<'s, 'vm> {
    pub(crate) fn new(
        scope: &'s Scope<'vm>,
        args: Vec<Value>,
        options: &'s VMOptions,
        modules: ModulesMap<'vm>,
    ) -> Self {
        Self {
            scope,
            frames: Default::default(),
            stack: VMStack::new(args.into_iter().map(|v| v.into()).collect()),
            options,
            modules,
        }
    }
}

#[allow(unused_variables, unused_mut)]
impl<'vm> Runner<'vm> for ProcessRunner<'_, 'vm> {
    runner_common!();

    fn update_scope<F>(&mut self, index: usize, mut update: F) -> Result<(), VMError>
    where
        F: FnMut(&mut Scope<'vm>) -> Result<(), VMError>,
    {
        Err(VMError::todo("Process does not implement `update_scope`"))
    }

    fn call_frame(&mut self, scope_index: usize) -> Result<(), VMError> {
        Err(VMError::todo("Process does not implement `call_frame`"))
    }

    fn call_frame_memo(&mut self, scope_index: usize) -> Result<(), VMError> {
        Err(VMError::todo(
            "Process does not implement `call_frame_memo`",
        ))
    }

    fn goto(&mut self, scope_id: usize, pc: usize) -> Result<(), VMError> {
        Err(VMError::todo("Process does not implement `goto`"))
    }

    fn send(&mut self, args: usize) -> Result<(), VMError> {
        Err(VMError::todo("Process does not implement `send`"))
    }

    fn receive(&mut self, args: usize) -> Result<(), VMError> {
        Err(VMError::todo("Process does not implement `receive`"))
    }

    fn broadcast(&mut self, args: BroadcastArgs) -> Result<(), VMError> {
        Err(VMError::todo("Process does not implement `broadcast`"))
    }

    fn spawn(&mut self, scope_id: usize, timeout: Option<usize>) -> Result<(), VMError> {
        Err(VMError::todo("Process does not implement `spawn`"))
    }

    fn call(
        &mut self,
        module: ResolvedModule<'vm>,
        func: &'vm str,
        args: usize,
    ) -> Result<Value, VMError> {
        Err(VMError::todo("Process does not implement `call`"))
    }

    fn call_extension(
        &mut self,
        module: ResolvedModule<'vm>,
        func: &'vm str,
        args: usize,
    ) -> Result<Value, VMError> {
        Err(VMError::todo("Process does not implement `call_extension`"))
    }

    fn call_mutable_extension(
        &mut self,
        module: ResolvedModule<'vm>,
        func: &'vm str,
        args: usize,
    ) -> Result<Option<Value>, VMError> {
        Err(VMError::todo(
            "Process does not implement `call_mutable_extension`",
        ))
    }

    fn vm_extension(
        &mut self,
        module: ResolvedModule<'vm>,
        func: &'vm str,
        args: usize,
    ) -> Result<Value, VMError> {
        Err(VMError::todo("Process does not implement `vm_extension`"))
    }
}

impl ProcessRunner<'_, '_> {
    pub fn run(&mut self) -> Value {
        for (arg, mutable) in &self.scope.args {
            let v = if *mutable {
                self.load_mut(arg)
            } else {
                self.load_let(arg)
            };
            if let Err(e) = v {
                return e.into();
            }
        }

        loop {
            let pc = self.frames.current.borrow().pc;
            if pc >= self.scope.instructions.len() {
                break;
            }
            let instruction = self.scope.instructions[pc].clone();
            self.frames.current.borrow_mut().pc += 1;
            let state: VMState = if let Instruction::Ret = instruction {
                VMState::Ran(self.stack.next_value("process_run").resolve(self))
            } else {
                self.process_core_instruction(instruction)
            };

            match state {
                VMState::Running => {}
                VMState::Done(v) | VMState::Ran(v) => return v.borrow().clone(),
            }
        }
        Value::Error(VMError::RuntimeError(
            "No return found in scope".to_string(),
        ))
    }
}
