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

impl ResolveValue for ProcessRunner<'_, '_> {
    fn location(&self) -> &'static str {
        "Process"
    }

    fn handle_scope(&mut self, scope: usize) -> Rc<RefCell<Value>> {
        todo!()
    }

    fn get_constant(&self, constant_id: usize) -> Rc<RefCell<Value>> {
        todo!()
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

impl<'vm> Runner<'vm> for ProcessRunner<'_, 'vm> {
    runner_common!();

    fn update_scope<F>(&mut self, index: usize, mut update: F) -> Result<(), VMError>
    where
        F: FnMut(&mut Scope<'vm>) -> Result<(), VMError>,
    {
        todo!()
    }

    fn call_frame(&mut self, scope_index: usize) -> Result<(), VMError> {
        todo!()
    }

    fn call_frame_memo(&mut self, scope_index: usize) -> Result<(), VMError> {
        todo!()
    }

    fn goto(&mut self, scope_id: usize, pc: usize) {
        todo!()
    }

    fn send(&mut self, args: usize) -> Result<(), VMState> {
        todo!()
    }

    fn receive(&mut self, args: usize) -> Result<(), VMState> {
        todo!()
    }

    fn broadcast(&mut self, args: BroadcastArgs) -> Result<(), VMState> {
        todo!()
    }

    fn spawn(&mut self, scope_id: usize, timeout: Option<usize>) -> Result<(), VMState> {
        todo!()
    }

    fn call(
        &mut self,
        module: ResolvedModule<'vm>,
        func: &'vm str,
        args: usize,
    ) -> Result<Value, VMError> {
        todo!()
    }

    fn call_extension(
        &mut self,
        module: ResolvedModule<'vm>,
        func: &'vm str,
        args: usize,
    ) -> Result<Value, VMError> {
        todo!()
    }

    fn call_mutable_extension(
        &mut self,
        module: ResolvedModule<'vm>,
        func: &'vm str,
        args: usize,
    ) -> Result<Option<Value>, VMError> {
        todo!()
    }

    fn vm_extension(
        &mut self,
        module: ResolvedModule<'vm>,
        func: &'vm str,
        args: usize,
    ) -> Result<Value, VMError> {
        todo!()
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
            let instruction = &self.scope.instructions[pc];
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
