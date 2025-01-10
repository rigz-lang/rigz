mod options;
mod runner;
mod values;

use crate::call_frame::Frames;
use crate::lifecycle::{Lifecycle, TestResults};
use crate::process::{ModulesMap, Process, SpawnedProcess};
use crate::{generate_builder, CallFrame, Instruction, Runner, Scope, VMStack, Variable};
use crate::{handle_js, out, outln, Module, RigzBuilder, VMError, Value};
use derive_more::IntoIterator;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

pub use options::VMOptions;
pub use values::*;

#[derive(Debug)]
pub struct VM<'vm> {
    pub scopes: Vec<Scope<'vm>>,
    pub frames: Frames<'vm>,
    pub modules: ModulesMap<'vm>,
    pub stack: VMStack,
    pub sp: usize,
    pub options: VMOptions,
    pub lifecycles: Vec<Lifecycle>,
    pub constants: Vec<Value>,
    pub(crate) processes: Vec<SpawnedProcess<'vm>>,
}

impl<'vm> RigzBuilder<'vm> for VM<'vm> {
    generate_builder!();

    #[inline]
    fn build(self) -> VM<'vm> {
        self
    }
}

impl Default for VM<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            scopes: vec![Scope::default()],
            frames: Default::default(),
            modules: Default::default(),
            sp: 0,
            options: Default::default(),
            lifecycles: Default::default(),
            constants: Default::default(),
            stack: Default::default(),
            processes: vec![],
        }
    }
}

impl<'vm> VM<'vm> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn from_scopes(scopes: Vec<Scope<'vm>>) -> Self {
        Self {
            scopes,
            ..Default::default()
        }
    }

    pub fn process_ret(&mut self, ran: bool) -> VMState {
        match self.frames.pop() {
            None => {
                let source = self.next_value("process_ret - empty stack");
                VMState::Done(source.resolve(self))
            }
            Some(c) => {
                let c = c;
                let pc = self.frames.current.borrow().pc;
                let mut updated = false;
                loop {
                    let sp = self.sp;
                    let scope = &self.scopes[sp];
                    let len = scope.instructions.len();
                    let propagate = len != pc && matches!(scope.named, "if" | "unless" | "else");
                    if propagate {
                        match self.frames.pop() {
                            None => {
                                let source = self.next_value("process_ret - empty stack");
                                return VMState::Done(source.resolve(self));
                            }
                            Some(next) => {
                                self.sp = next.borrow().scope_id;
                                self.frames.current = next;
                                updated = true;
                            }
                        }
                    } else {
                        break;
                    }
                }
                if !updated {
                    self.sp = c.borrow().scope_id;
                    self.frames.current = c;
                }
                match ran {
                    false => VMState::Running,
                    true => {
                        let source = self.next_value("process_ret - ran");
                        VMState::Ran(source.resolve(self))
                    }
                }
            }
        }
    }

    #[inline]
    fn process_instruction(&mut self, instruction: Instruction<'vm>) -> VMState {
        match instruction {
            Instruction::Ret => self.process_ret(false),
            instruction => self.process_core_instruction(instruction),
        }
    }

    fn process_instruction_scope(&mut self, instruction: Instruction<'vm>) -> VMState {
        match instruction {
            Instruction::Ret => self.process_ret(true),
            ins => self.process_core_instruction(ins),
        }
    }

    #[inline]
    fn next_instruction(&self) -> Option<Instruction<'vm>> {
        let scope = &self.scopes[self.sp];
        let pc = self.frames.current.borrow().pc;
        self.frames.current.borrow_mut().pc += 1;
        scope.instructions.get(pc).cloned()
    }

    /// Calls run and returns an error if the resulting value is an error
    pub fn eval(&mut self) -> Result<Value, VMError> {
        match self.run() {
            Value::Error(e) => Err(e),
            v => Ok(v),
        }
    }

    pub fn eval_within(&mut self, duration: Duration) -> Result<Value, VMError> {
        match self.run_within(duration) {
            Value::Error(e) => Err(e),
            v => Ok(v),
        }
    }

    pub fn add_bindings(&mut self, bindings: HashMap<&'vm str, (StackValue, bool)>) {
        let mut current = self.frames.current.borrow_mut();
        for (k, (v, mutable)) in bindings {
            let v = if mutable {
                Variable::Mut(v)
            } else {
                Variable::Let(v)
            };
            current.variables.insert(k, v);
        }
    }

    /// Starts processes for each "On" lifecycle, Errors are returned as Value::Error(VMError)
    pub fn run(&mut self) -> Value {
        self.start_processes();

        let mut run = || loop {
            match self.step() {
                None => {}
                Some(v) => return v,
            }
        };

        let res = run();
        self.close_processes(res)
    }

    #[inline]
    fn step(&mut self) -> Option<Value> {
        let instruction = match self.next_instruction() {
            // TODO this should probably be an error requiring explicit halt, result would be none
            None => return self.stack.pop().map(|e| e.resolve(self).borrow().clone()),
            Some(s) => s,
        };

        match self.process_instruction(instruction) {
            VMState::Ran(v) => {
                return Some(
                    VMError::RuntimeError(format!("Unexpected ran state: {}", v.borrow())).into(),
                )
            }
            VMState::Running => {}
            VMState::Done(v) => return Some(v.borrow().clone()),
        };
        None
    }

    fn start_processes(&mut self) {
        self.processes = self
            .scopes
            .iter()
            .filter(|s| matches!(s.lifecycle, Some(Lifecycle::On(_))))
            .map(|s| Process::spawn(s.clone(), self.options, self.modules.clone(), None))
            .collect();
    }

    fn close_processes(&mut self, result: Value) -> Value {
        let mut errors: Vec<VMError> = vec![];
        for p in self.processes.drain(..) {
            match p.close() {
                Ok(_) => {}
                Err(r) => {
                    errors.push(r);
                }
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
            VMError::RuntimeError(format!("Process Failures: {messages}")).into()
        }
    }

    pub fn run_within(&mut self, duration: Duration) -> Value {
        #[cfg(not(feature = "js"))]
        let now = std::time::Instant::now();
        #[cfg(feature = "js")]
        let now = web_time::Instant::now();
        loop {
            let elapsed = now.elapsed();
            if elapsed > duration {
                return VMError::TimeoutError(format!(
                    "Exceeded runtime {duration:?} - {:?}",
                    elapsed
                ))
                .into();
            }

            match self.step() {
                None => {}
                Some(v) => return v,
            }
        }
    }

    pub fn test(&mut self) -> TestResults {
        // todo support parallel tests
        let test_scopes: Vec<_> = self
            .scopes
            .iter()
            .enumerate()
            .filter_map(|(index, s)| match &s.lifecycle {
                None => None,
                Some(Lifecycle::Test(_)) => {
                    let Instruction::Ret =
                        s.instructions.last().expect("No instructions for scope")
                    else {
                        unreachable!("Invalid Scope")
                    };
                    Some((index, s.named))
                }
                Some(_) => None,
            })
            .collect();

        let mut passed = 0;
        let mut failed = 0;
        #[cfg(not(feature = "js"))]
        let start = std::time::Instant::now();
        #[cfg(feature = "js")]
        let start = web_time::Instant::now();
        let mut failure_messages = Vec::new();
        for (s, named) in test_scopes {
            out!("test {named} ... ");
            self.sp = s;
            self.frames.current = RefCell::new(CallFrame {
                scope_id: s,
                ..Default::default()
            });
            let v = self.eval();
            match v {
                Err(e) => {
                    outln!("\x1b[31mFAILED\x1b[0m");
                    failed += 1;
                    failure_messages.push((named.to_string(), e));
                }
                Ok(_) => {
                    outln!("\x1b[32mok\x1b[0m");
                    passed += 1;
                }
            };
        }

        TestResults {
            passed,
            failed,
            failure_messages,
            duration: start.elapsed(),
        }
    }

    pub fn run_scope(&mut self) -> VMState {
        loop {
            let instruction = match self.next_instruction() {
                // TODO this should probably be an error requiring explicit halt, result would be none
                None => return VMState::Done(Value::None.into()),
                Some(s) => s,
            };

            match self.process_instruction_scope(instruction) {
                VMState::Running => {}
                s => return s,
            };
        }
    }

    /// All variables are reset and will need to be set again by calling `add_bindings`
    pub fn reset(&mut self) {
        self.sp = 0;
        self.stack.clear();
        self.frames.reset()
    }

    /// Snapshots can't include modules or messages from in progress lifecycles
    pub fn snapshot(&self) -> Result<Vec<u8>, VMError> {
        let mut bytes = Vec::new();
        bytes.push(self.options.as_byte());
        bytes.extend((self.sp as u64).to_le_bytes());

        // write registers
        // write stack
        // write scopes
        // write current
        // write call_frames
        Ok(bytes)
    }

    /// Snapshots can't include modules so VM must be created before loading snapshot
    pub fn load_snapshot(&mut self, bytes: Vec<u8>) -> Result<(), VMError> {
        let mut bytes = bytes.into_iter();
        self.options = VMOptions::from_byte(bytes.next().unwrap());
        let mut sp = [0; 8];
        for (i, b) in bytes.take(8).enumerate() {
            sp[i] = b;
        }
        self.sp = u64::from_le_bytes(sp) as usize;
        // load registers
        // load stack
        // load scopes
        // load current
        // load call_frames
        Ok(())
    }
}

#[cfg(test)]
pub mod vm_tests {
    use crate::builder::RigzBuilder;
    use crate::vm::VM;
    use crate::{VMBuilder, Value};
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test(unsupported = test)]
    fn snapshot() {
        let mut builder = VMBuilder::new();
        builder.add_load_instruction(Value::Bool(true).into());
        let vm = builder.build();
        let bytes = vm.snapshot().expect("snapshot failed");
        let mut vm2 = VM::default();
        vm2.load_snapshot(bytes).expect("load failed");
        assert_eq!(vm2.options, vm.options);
        assert_eq!(vm2.sp, vm.sp);
        // assert_eq!(vm2.get_register(1), Value::Bool(true).into());
    }
}
