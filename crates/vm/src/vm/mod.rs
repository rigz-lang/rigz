mod options;
mod runner;
mod values;

use crate::call_frame::Frames;
use crate::process::ProcessManager;
use crate::{
    generate_builder, out, CallFrame, Instruction, RigzBuilder, Runner, Scope, VMStack, Variable,
};
pub use options::VMOptions;
use rigz_core::{
    Dependency, EnumDeclaration, Lifecycle, Module, MutableReference, ObjectValue, PrimitiveValue,
    Snapshot, StackValue, TestResults, VMError,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::time::Duration;
pub use values::*;

#[cfg(feature = "threaded")]
pub type ModulesMap =
    std::sync::Arc<dashmap::DashMap<&'static str, std::sync::Arc<dyn Module + Send + Sync>>>;

#[cfg(not(feature = "threaded"))]
pub type ModulesMap = HashMap<&'static str, std::rc::Rc<dyn Module>>;

pub type Dependencies = RwLock<Vec<Arc<Dependency>>>;

#[derive(Debug)]
pub struct VM {
    pub scopes: Vec<Scope>,
    pub frames: Frames,
    pub modules: ModulesMap,
    pub(crate) dependencies: Dependencies,
    pub stack: VMStack,
    pub sp: usize,
    pub options: VMOptions,
    pub lifecycles: Vec<Lifecycle>,
    pub constants: Vec<ObjectValue>,
    pub(crate) process_manager: MutableReference<ProcessManager>,
    pub enums: RwLock<Vec<Arc<EnumDeclaration>>>,
}

impl RigzBuilder for VM {
    generate_builder!();

    #[inline]
    fn build(self) -> VM {
        self
    }

    #[inline]
    fn register_dependency(&mut self, dependency: Arc<Dependency>) -> usize {
        let dep = self
            .dependencies
            .read()
            .expect("failed to read dependencies")
            .len();
        self.dependencies
            .get_mut()
            .expect("failed to lock dependencies")
            .push(dependency);
        dep
    }

    #[inline]
    fn register_enum(&mut self, decl: Arc<EnumDeclaration>) -> usize {
        let dep = self.enums.read().expect("failed to read enums").len();
        self.enums
            .get_mut()
            .expect("failed to lock enums")
            .push(decl);
        dep
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
            dependencies: Default::default(),
            enums: Default::default(),
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
        let mut last_scope = self.sp;
        match self.frames.pop() {
            None => {
                let source = self.next_value("process_ret - empty stack");
                VMState::Done(source.resolve(self))
            }
            Some(c) => {
                let c = c;
                let mut updated = false;
                loop {
                    let sp = self.frames.current.borrow().scope_id;
                    let pc = self.frames.current.borrow().pc;
                    let scope = &self.scopes[sp];
                    let len = scope.instructions.len();
                    let propagate = len != pc
                        && matches!(
                            scope.named.as_str(),
                            "if" | "unless" | "else" | "loop" | "for"
                        );
                    if propagate {
                        match self.frames.pop() {
                            None => {
                                let source = self.next_value("process_ret - empty stack");
                                return VMState::Done(source.resolve(self));
                            }
                            Some(next) => {
                                last_scope = c.borrow().scope_id;
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
                        let source = self.next_resolved_value("process_ret - ran");
                        if updated
                            && matches!(
                                self.scopes[last_scope].named.as_str(),
                                "if" | "unless" | "else" | "loop" | "for"
                            )
                        {
                            VMState::Done(source)
                        } else {
                            VMState::Ran(source)
                        }
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
        let sp = self.frames.current.borrow().scope_id;
        let scope = &self.scopes[sp];
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
        self.run_within(duration).map_err(|e| e.into())
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
            VMState::Break => {
                return Some(
                    VMError::UnsupportedOperation("Invalid break instruction".to_string()).into(),
                )
            }
            VMState::Next => {
                return Some(
                    VMError::UnsupportedOperation("Invalid next instruction".to_string()).into(),
                )
            }
            VMState::Running => {}
            VMState::Ran(v) | VMState::Done(v) => return Some(v.borrow().clone()),
        };
        None
    }

    pub fn run_within(&mut self, duration: Duration) -> Result<ObjectValue, VMError> {
        self.start_processes();
        #[cfg(not(feature = "js"))]
        let now = std::time::Instant::now();
        #[cfg(feature = "js")]
        let now = web_time::Instant::now();
        let mut run = || loop {
            let elapsed = now.elapsed();
            if elapsed > duration {
                // todo this should be a dedicated error type so it's not misused
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
        // todo this needs to be pause processes if timeout error was hit
        match self.process_manager.update(move |p| p.close(res)) {
            ObjectValue::Primitive(PrimitiveValue::Error(e)) => Err(e),
            o => Ok(o),
        }
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
