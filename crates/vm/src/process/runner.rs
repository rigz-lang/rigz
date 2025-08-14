use crate::call_frame::{CallFrame, Frames};
use crate::process::ProcessManager;
use crate::{
    runner_common, CallType, Instruction, Modules, ResolvedModule, Runner, Scope, VMOptions,
    VMStack, VMState, Variable,
};
use log_derive::{logfn, logfn_inputs};
use rigz_core::{
    EnumDeclaration, MutableReference, ObjectValue, ResolveValue, ResolvedValue, RigzArgs,
    StackValue, VMError,
};
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
    modules: Modules,
    process_manager: MutableReference<ProcessManager>,
}

#[allow(unused_variables)]
impl ResolveValue for ProcessRunner<'_> {
    fn location(&self) -> &'static str {
        "Process"
    }

    fn handle_scope(&mut self, scope: usize) -> ResolvedValue {
        let o: ObjectValue = VMError::todo("Process does not implement `handle_scope`").into();
        ResolvedValue::Value(o.into())
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
        modules: Modules,
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

    fn find_enum(&mut self, enum_type: usize) -> Result<std::sync::Arc<EnumDeclaration>, VMError> {
        Err(VMError::todo("Process does not implement `find_enum`"))
    }

    fn call(
        &mut self,
        module: ResolvedModule,
        func: &str,
        args: usize,
    ) -> Result<ObjectValue, VMError> {
        Err(VMError::todo("Process does not implement `call`"))
    }

    fn call_loop(&mut self, scope_index: usize) -> Option<VMState> {
        Some(VMError::todo("Process does not implement `call_loop`").into())
    }

    fn call_for(&mut self, scope_index: usize) -> Option<VMState> {
        Some(VMError::todo("Process does not implement `call_for`").into())
    }

    fn call_for_comprehension<T, I, F>(
        &mut self,
        scope_id: usize,
        init: I,
        mut save: F,
    ) -> Result<T, VMState>
    where
        F: FnMut(&mut T, ObjectValue) -> Option<VMError>,
        I: FnOnce(usize) -> T,
    {
        Err(VMError::todo("Process does not implement `call_for_comprehension`").into())
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
        std::thread::sleep(duration)
    }

    fn exit<V>(&mut self, value: V)
    where
        V: Into<StackValue>,
    {
        self.stack.push(value.into());
        self.frames.current.borrow_mut().pc = self.scope.instructions.len() - 1;
    }
}

impl ProcessRunner<'_> {
    #[inline]
    fn load_args(&mut self) -> Result<(), VMError> {
        for (arg, mutable) in self.scope.args.clone() {
            if mutable {
                self.load_mut(arg, false)?
            } else {
                self.load_let(arg, false)?
            };
        }
        Ok(())
    }

    #[inline]
    fn step(&mut self, pc: usize) -> Option<ObjectValue> {
        let instruction = self.scope.instructions[pc].clone();
        self.frames.current.borrow_mut().pc += 1;
        let state: VMState = if let Instruction::Ret = instruction {
            VMState::Ran(self.stack.next_value(|| "process_run").resolve(self))
        } else {
            self.process_core_instruction(&instruction)
        };

        match state {
            VMState::Break => {
                Some(VMError::UnsupportedOperation("Invalid break instruction".to_string()).into())
            }
            VMState::Next => {
                Some(VMError::UnsupportedOperation("Invalid next instruction".to_string()).into())
            }
            VMState::Running => None,
            VMState::Done(v) | VMState::Ran(v) => Some(v.borrow().clone()),
        }
    }

    #[cfg(not(feature = "threaded"))]
    pub(crate) fn run_within(&mut self, timeout: usize) -> ObjectValue {
        let until = Duration::from_millis(timeout as u64);
        #[cfg(feature = "js")]
        let now = web_time::Instant::now();
        #[cfg(not(feature = "js"))]
        let now = std::time::Instant::now();
        if let Err(e) = self.load_args() {
            return e.into();
        }

        loop {
            if now.elapsed() >= until {
                return VMError::runtime(format!("`receive` timed out after {:?}ms", timeout))
                    .into();
            }

            let pc = self.frames.current.borrow().pc;
            if pc >= self.scope.instructions.len() {
                break;
            }

            if let Some(v) = self.step(pc) {
                return v;
            }
        }
        VMError::runtime("No return found in scope".to_string()).into()
    }

    pub fn run(&mut self) -> ObjectValue {
        if let Err(e) = self.load_args() {
            return e.into();
        }

        loop {
            let pc = self.frames.current.borrow().pc;
            if pc >= self.scope.instructions.len() {
                break;
            }

            if let Some(v) = self.step(pc) {
                return v;
            }
        }
        VMError::runtime("No return found in scope".to_string()).into()
    }
}
