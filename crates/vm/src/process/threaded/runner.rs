use crate::call_frame::{CallFrame, Frames};
use crate::process::ModulesMap;
use crate::{
    runner_common, Instruction, ResolveValue, ResolvedModule, Runner, Scope, StackValue, VMError,
    VMOptions, VMStack, VMState, Value, Variable,
};
use log_derive::{logfn, logfn_inputs};
use std::cell::RefCell;
use std::fmt::Display;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Duration;
use tokio::runtime::Handle;

pub(crate) struct ProcessRunner<'s> {
    scope: &'s Scope,
    frames: Frames,
    stack: VMStack,
    options: &'s VMOptions,
    modules: ModulesMap,
    handle: Handle,
}

#[allow(unused_variables)]
impl ResolveValue for ProcessRunner<'_> {
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

impl<'s> ProcessRunner<'s> {
    pub(crate) fn new(
        scope: &'s Scope,
        args: Vec<Value>,
        options: &'s VMOptions,
        modules: ModulesMap,
        handle: Handle,
    ) -> Self {
        Self {
            scope,
            frames: Default::default(),
            stack: VMStack::new(args.into_iter().map(|v| v.into()).collect()),
            options,
            modules,
            handle,
        }
    }
}

#[allow(unused_variables, unused_mut)]
impl Runner for ProcessRunner<'_> {
    runner_common!();

    fn update_scope<F>(&mut self, index: usize, mut update: F) -> Result<(), VMError>
    where
        F: FnMut(&mut Scope) -> Result<(), VMError>,
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

    fn broadcast(&mut self, args: usize) -> Result<(), VMError> {
        Err(VMError::todo("Process does not implement `broadcast`"))
    }

    fn spawn(&mut self, scope_id: usize, timeout: Option<usize>) -> Result<(), VMError> {
        Err(VMError::todo("Process does not implement `spawn`"))
    }

    fn call(
        &mut self,
        module: ResolvedModule,
        func: String,
        args: usize,
    ) -> Result<Value, VMError> {
        Err(VMError::todo("Process does not implement `call`"))
    }

    fn vm_extension(
        &mut self,
        module: ResolvedModule,
        func: String,
        args: usize,
    ) -> Result<Value, VMError> {
        Err(VMError::todo("Process does not implement `vm_extension`"))
    }

    fn sleep(&self, duration: Duration) {
        self.handle.block_on(tokio::time::sleep(duration));
    }
}

impl ProcessRunner<'_> {
    pub fn run(&mut self) -> Value {
        for (arg, mutable) in self.scope.args.clone() {
            let v = if mutable {
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
