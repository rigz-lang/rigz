use crate::call_frame::{CallFrame, Frames};
use crate::process::ProcessManager;
use crate::{
    runner_common, CallType, Instruction, ModulesMap, ResolvedModule, Runner, Scope, VMOptions,
    VMStack, VMState, Variable,
};
use log_derive::{logfn, logfn_inputs};
use rigz_core::{EnumDeclaration, MutableReference, ObjectValue, ResolveValue, RigzArgs, StackValue, VMError};
use std::cell::RefCell;
use std::fmt::Display;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Duration;

pub(crate) struct ProcessRunner<'s> {
    scope: &'s Scope,
    frames: Frames,
    stack: VMStack,
    options: &'s VMOptions,
    modules: ModulesMap,
    process_manager: MutableReference<ProcessManager>,
}

#[allow(unused_variables)]
impl ResolveValue for ProcessRunner<'_> {
    fn location(&self) -> &'static str {
        "Process"
    }

    fn handle_scope(&mut self, scope: usize) -> Rc<RefCell<ObjectValue>> {
        let o: ObjectValue = VMError::todo("Process does not implement `handle_scope`").into();
        o.into()
    }

    fn get_constant(&self, constant_id: usize) -> Rc<RefCell<ObjectValue>> {
        let o: ObjectValue = VMError::todo("Process does not implement `get_constant`").into();
        o.into()
    }
}

impl<'s> ProcessRunner<'s> {
    pub(crate) fn new(
        scope: &'s Scope,
        args: Vec<ObjectValue>,
        options: &'s VMOptions,
        modules: ModulesMap,
        process_manager: MutableReference<ProcessManager>,
    ) -> Self {
        Self {
            scope,
            frames: Default::default(),
            stack: VMStack::new(args.into_iter().map(|v| v.into()).collect()),
            options,
            modules,
            process_manager,
        }
    }
}

#[allow(unused_variables, unused_mut)]
impl Runner for ProcessRunner<'_> {
    runner_common!();

    fn find_enum(&mut self, enum_type: usize) -> Result<std::sync::Arc<EnumDeclaration>, VMError> {
        Err(VMError::todo("Process does not implement `find_enum`"))
    }

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

    fn spawn(&mut self, scope_id: usize, timeout: Option<usize>) -> Result<(), VMError> {
        Err(VMError::todo("Process does not implement `spawn`"))
    }

    fn call(
        &mut self,
        module: ResolvedModule,
        func: String,
        args: usize,
    ) -> Result<ObjectValue, VMError> {
        Err(VMError::todo("Process does not implement `call`"))
    }

    // fn vm_extension(
    //     &mut self,
    //     module: ResolvedModule,
    //     func: String,
    //     args: usize,
    // ) -> Result<ObjectValue, VMError> {
    //     Err(VMError::todo("Process does not implement `vm_extension`"))
    // }

    fn sleep(&self, duration: Duration) {
        #[cfg(feature = "threaded")]
        self.process_manager
            .apply(move |pm| pm.handle.block_on(tokio::time::sleep(duration)));

        #[cfg(not(feature = "threaded"))]
        thread::sleep(duration)
    }

    fn call_dependency(
        &mut self,
        arg: RigzArgs,
        dep: usize,
        call_type: CallType,
    ) -> Result<ObjectValue, VMError> {
        Err(VMError::todo(format!(
            "Process does not implement call dependency {dep}"
        )))
    }
}

impl ProcessRunner<'_> {
    pub fn run(&mut self) -> ObjectValue {
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
        VMError::RuntimeError("No return found in scope".to_string()).into()
    }
}
