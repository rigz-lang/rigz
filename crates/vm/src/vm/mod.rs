mod options;
mod runner;
mod values;

use crate::call_frame::Frames;
use crate::process::{ModulesMap, ProcessManager};
use crate::{
    generate_builder, out, CallFrame, Instruction, RigzBuilder, Runner, Scope, VMStack, Variable,
};
pub use options::VMOptions;
use rigz_core::{
    Lifecycle, Module, MutableReference, ObjectValue, PrimitiveValue, Snapshot, StackValue,
    TestResults, VMError,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;
pub use values::*;

#[derive(Debug)]
pub struct VM {
    pub scopes: Vec<Scope>,
    pub frames: Frames,
    pub modules: ModulesMap,
    pub stack: VMStack,
    pub sp: usize,
    pub options: VMOptions,
    pub lifecycles: Vec<Lifecycle>,
    pub constants: Vec<ObjectValue>,
    pub(crate) process_manager: MutableReference<ProcessManager>,
}

impl RigzBuilder for VM {
    generate_builder!();

    #[inline]
    fn build(self) -> VM {
        self
    }
}

impl Default for VM {
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
            #[cfg(feature = "threaded")]
            process_manager: ProcessManager::create()
                .expect("Failed to setup ProcessManager")
                .into(),
            #[cfg(not(feature = "threaded"))]
            process_manager: ProcessManager::new().into(),
        }
    }
}

impl VM {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn from_scopes(scopes: Vec<Scope>) -> Self {
        Self {
            scopes,
            ..Default::default()
        }
    }

    #[inline]
    pub fn from_modules(modules: ModulesMap) -> Self {
        Self {
            modules,
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
                    let propagate =
                        len != pc && matches!(scope.named.as_str(), "if" | "unless" | "else");
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
    fn process_instruction(&mut self, instruction: Instruction) -> VMState {
        match instruction {
            Instruction::Ret => self.process_ret(false),
            instruction => self.process_core_instruction(instruction),
        }
    }

    fn process_instruction_scope(&mut self, instruction: Instruction) -> VMState {
        match instruction {
            Instruction::Ret => self.process_ret(true),
            ins => self.process_core_instruction(ins),
        }
    }

    #[inline]
    fn next_instruction(&self) -> Option<Instruction> {
        let scope = &self.scopes[self.sp];
        let pc = self.frames.current.borrow().pc;
        self.frames.current.borrow_mut().pc += 1;
        scope.instructions.get(pc).cloned()
    }

    /// Calls run and returns an error if the resulting value is an error
    pub fn eval(&mut self) -> Result<ObjectValue, VMError> {
        match self.run() {
            ObjectValue::Primitive(PrimitiveValue::Error(e)) => Err(e),
            v => Ok(v),
        }
    }

    pub fn eval_within(&mut self, duration: Duration) -> Result<ObjectValue, VMError> {
        match self.run_within(duration) {
            ObjectValue::Primitive(PrimitiveValue::Error(e)) => Err(e),
            v => Ok(v),
        }
    }

    pub fn add_bindings(&mut self, bindings: HashMap<String, (StackValue, bool)>) {
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
    pub fn run(&mut self) -> ObjectValue {
        self.start_processes();

        let mut run = || loop {
            if let Some(v) = self.step() {
                return v;
            }
        };

        let res = run();
        self.process_manager.update(move |r| r.close(res))
    }

    #[inline]
    fn step(&mut self) -> Option<ObjectValue> {
        let instruction = match self.next_instruction() {
            // TODO this should probably be an error requiring explicit halt, this might still be an error
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

    pub fn run_within(&mut self, duration: Duration) -> ObjectValue {
        self.start_processes();
        #[cfg(not(feature = "js"))]
        let now = std::time::Instant::now();
        #[cfg(feature = "js")]
        let now = web_time::Instant::now();
        let mut run = || loop {
            let elapsed = now.elapsed();
            if elapsed > duration {
                return VMError::TimeoutError(format!(
                    "Exceeded runtime {duration:?} - {:?}",
                    elapsed
                ))
                .into();
            }

            if let Some(v) = self.step() {
                return v;
            }
        };
        let res = run();
        self.process_manager.update(move |p| p.close(res))
    }

    fn start_processes(&mut self) {
        let processes = ProcessManager::create_on_processes(self);
        self.process_manager.update(move |p| p.add(processes));
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
                    Some((index, s.named.clone()))
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
                    #[cfg(not(feature = "js"))]
                    println!("\x1b[31mFAILED\x1b[0m");
                    #[cfg(feature = "js")]
                    web_sys::console::log_2(&"%c FAILED".into(), &"color: red".into());
                    failed += 1;
                    failure_messages.push((named.to_string(), e));
                }
                Ok(_) => {
                    #[cfg(not(feature = "js"))]
                    println!("\x1b[32mok\x1b[0m");
                    #[cfg(feature = "js")]
                    web_sys::console::log_2(&"%c ok".into(), &"color: green".into());
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
                None => return VMState::Done(ObjectValue::default().into()),
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

    /// Snapshots won't include modules
    pub fn snapshot(&self) -> Result<Vec<u8>, VMError> {
        let mut bytes = Vec::new();
        bytes.extend(self.options.as_bytes());
        bytes.extend(self.sp.as_bytes());
        bytes.extend(self.stack.as_bytes());
        bytes.extend(self.scopes.as_bytes());
        bytes.extend(self.frames.as_bytes());
        bytes.extend(self.lifecycles.as_bytes());
        bytes.extend(self.constants.as_bytes());
        Ok(bytes)
    }

    /// Snapshots can't include modules so VM must be created before loading snapshot, missing modules will fail at runtime
    pub fn load_snapshot(&mut self, bytes: Vec<u8>) -> Result<(), VMError> {
        let mut bytes = bytes.into_iter();
        self.options = VMOptions::from_bytes(&mut bytes, &"load snapshot: vm options")?;
        self.sp = Snapshot::from_bytes(&mut bytes, &"load snapshot: sp")?;
        self.stack = Snapshot::from_bytes(&mut bytes, &"load snapshot: stack")?;
        self.scopes = Snapshot::from_bytes(&mut bytes, &"load snapshot: scopes")?;
        self.frames = Snapshot::from_bytes(&mut bytes, &"load snapshot: frames")?;
        self.lifecycles = Snapshot::from_bytes(&mut bytes, &"load snapshot: lifecycles")?;
        self.constants = Snapshot::from_bytes(&mut bytes, &"load snapshot: constants")?;
        Ok(())
    }
}

#[cfg(test)]
pub mod vm_tests {
    use crate::builder::RigzBuilder;
    use crate::vm::VM;
    use crate::VMBuilder;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test(unsupported = test)]
    fn snapshot() {
        let mut builder = VMBuilder::new();
        builder.add_load_instruction(true.into());
        let vm = builder.build();
        let bytes = vm.snapshot().expect("snapshot failed");
        let mut vm2 = VM::default();
        vm2.load_snapshot(bytes).expect("load failed");
        assert_eq!(vm2.options, vm.options);
        assert_eq!(vm2.sp, vm.sp);
        assert_eq!(vm2.scopes, vm.scopes);
        assert_eq!(vm2.frames, vm.frames);
        assert_eq!(vm2.lifecycles, vm.lifecycles);
    }
}
