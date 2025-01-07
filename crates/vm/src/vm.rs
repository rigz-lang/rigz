use crate::call_frame::Frames;
use crate::lifecycle::{Lifecycle, TestResults};
use crate::process::Process;
use crate::{generate_builder, CallFrame, Instruction, Scope, Variable};
use crate::{out, outln, Module, RigzBuilder, VMError, Value};
use indexmap::IndexMap;
use itertools::Itertools;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::ptr;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub enum VMState {
    Running,
    Done(Rc<RefCell<Value>>),
    Ran(Rc<RefCell<Value>>),
}

impl From<VMError> for VMState {
    #[inline]
    fn from(value: VMError) -> Self {
        VMState::Done(value.into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VMOptions {
    pub enable_logging: bool,
    pub disable_modules: bool,
    pub disable_variable_cleanup: bool,
    pub max_depth: usize,
}

impl Default for VMOptions {
    fn default() -> Self {
        VMOptions {
            enable_logging: true,
            disable_modules: false,
            disable_variable_cleanup: false,
            max_depth: 1024,
        }
    }
}

impl VMOptions {
    fn to_byte(&self) -> u8 {
        let mut result = 0;
        result |= self.enable_logging as u8;
        result |= (self.disable_modules as u8) << 1;
        result |= (self.disable_variable_cleanup as u8) << 2;
        result
    }

    fn from_byte(byte: u8) -> Self {
        VMOptions {
            enable_logging: (byte & 1) == 1,
            disable_modules: (byte & 1 << 1) == 2,
            disable_variable_cleanup: (byte & 1 << 2) == 4,
            max_depth: 1024,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StackValue {
    ScopeId(usize),
    Value(Rc<RefCell<Value>>),
    Constant(usize),
}

impl From<Rc<RefCell<Value>>> for StackValue {
    #[inline]
    fn from(value: Rc<RefCell<Value>>) -> Self {
        StackValue::Value(value)
    }
}

impl StackValue {
    pub fn resolve(&self, vm: &mut VM) -> Rc<RefCell<Value>> {
        match self {
            &StackValue::ScopeId(scope) => vm.handle_scope(scope),
            StackValue::Value(v) => v.clone(),
            &StackValue::Constant(c) => vm.get_constant(c),
        }
    }
}

impl<T: Into<Value>> From<T> for StackValue {
    #[inline]
    fn from(value: T) -> Self {
        StackValue::Value(Rc::new(RefCell::new(value.into())))
    }
}

#[derive(Clone, Debug)]
pub struct VM<'vm> {
    pub scopes: Vec<Scope<'vm>>,
    pub frames: Frames<'vm>,
    pub modules: IndexMap<&'static str, Rc<RefCell<dyn Module<'vm>>>>,
    pub stack: Vec<StackValue>,
    pub sp: usize,
    pub options: VMOptions,
    pub lifecycles: Vec<Lifecycle>,
    pub constants: Vec<Value>,
    pub processes: Vec<Process<'vm>>,
}

impl<'v> VM<'v> {
    pub fn next_value<T: Display>(&mut self, location: T) -> StackValue {
        self.stack
            .pop()
            .unwrap_or_else(|| VMError::EmptyStack(format!("Stack is empty for {location}")).into())
    }

    pub fn next_resolved_value<T: Display>(&mut self, location: T) -> Rc<RefCell<Value>> {
        self.stack
            .pop()
            .unwrap_or_else(|| VMError::EmptyStack(format!("Stack is empty for {location}")).into())
            .resolve(self)
    }

    pub fn store_value(&mut self, value: StackValue) {
        self.stack.push(value)
    }

    pub fn resolve_args(&mut self, count: usize) -> Vec<Rc<RefCell<Value>>> {
        (0..count)
            .map(|_| self.next_resolved_value("resolve_args"))
            .collect()
    }

    pub fn get_variable(&mut self, name: &'v str) -> Result<Rc<RefCell<Value>>, VMError> {
        let v = match self.frames.current.borrow().get_variable(name, self) {
            None => {
                return Err(VMError::VariableDoesNotExist(format!(
                    "Immutable variable {name} does not exist"
                )))
            }
            Some(v) => v,
        };
        Ok(v.resolve(self))
    }

    pub fn get_mutable_variable(&mut self, name: &'v str) -> Result<Rc<RefCell<Value>>, VMError> {
        let v = match self
            .frames
            .current
            .borrow()
            .get_mutable_variable(name, self)?
        {
            None => {
                return Err(VMError::VariableDoesNotExist(format!(
                    "Mutable variable {name} does not exist"
                )))
            }
            Some(v) => v,
        };
        Ok(v.resolve(self))
    }
}

impl<'vm> RigzBuilder<'vm> for VM<'vm> {
    generate_builder!();

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

// todo all functions that return results should be mapped to Value::Error instead (when possible)
impl<'vm> VM<'vm> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_module_clone(
        &self,
        module: &'vm str,
    ) -> Result<Rc<RefCell<dyn Module<'vm>>>, VMError> {
        match self.modules.get(&module) {
            None => Err(VMError::InvalidModule(module.to_string())),
            Some(m) => Ok(m.clone()),
        }
    }

    #[inline]
    pub fn handle_scope(&mut self, scope: usize) -> Rc<RefCell<Value>> {
        let current = self.sp;
        match self.call_frame(scope) {
            Ok(_) => {}
            Err(e) => return e.into(),
        };
        let mut v = match self.run_scope() {
            VMState::Running => unreachable!(),
            VMState::Done(v) => return v,
            VMState::Ran(v) => v,
        };
        while current != self.sp {
            self.stack.push(v.into());
            v = match self.run_scope() {
                VMState::Running => unreachable!(),
                VMState::Done(v) => return v,
                VMState::Ran(v) => v,
            };
        }
        v
    }

    pub fn get_constant(&self, index: usize) -> Rc<RefCell<Value>> {
        match self.constants.get(index) {
            None => VMError::RuntimeError(format!("Constant {index} does not exist")).into(),
            Some(v) => v.clone().into(),
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
    fn process_instruction(&mut self, instruction: *const Instruction<'vm>) -> VMState {
        unsafe {
            match instruction.as_ref().unwrap() {
                Instruction::Ret => self.process_ret(false),
                instruction => self.process_core_instruction(instruction),
            }
        }
    }

    fn process_instruction_scope(&mut self, instruction: *const Instruction<'vm>) -> VMState {
        unsafe {
            match instruction.as_ref().unwrap() {
                Instruction::Ret => self.process_ret(true),
                ins => self.process_core_instruction(ins),
            }
        }
    }

    #[inline]
    fn next_instruction(&self) -> Option<*const Instruction<'vm>> {
        let scope_id = self.sp;
        // scope_id must be valid when this is called, otherwise function will panic
        let scope = &self.scopes[scope_id];
        let pc = self.frames.current.borrow().pc;
        self.frames.current.borrow_mut().pc += 1;
        scope.instructions.get(pc).map(ptr::from_ref)
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

    // Starts processes for each "On" lifecycle
    pub fn run(&mut self) -> Value {
        self.processes.extend(
            self.scopes
                .iter()
                .filter(|s| matches!(s.lifecycle, Some(Lifecycle::On(_))))
                .map(|s| Process::new(s.clone())),
        );

        self.processes.iter_mut().for_each(|p| p.start());

        let mut run = || loop {
            match self.step() {
                None => {}
                Some(v) => return v,
            }
        };

        let res = run();
        self.processes.iter_mut().for_each(|p| p.close());
        res
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

    pub fn run_within(&mut self, duration: Duration) -> Value {
        let now = Instant::now();
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
        let start = Instant::now();
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

    pub fn load_mut(&mut self, name: &'vm str) -> Result<(), VMError> {
        let v = self.next_value(format!("load_mut - {name}"));
        self.frames.load_mut(name, v)
    }

    pub fn load_let(&mut self, name: &'vm str) -> Result<(), VMError> {
        let v = self.next_value(format!("load_let - {name}"));
        self.frames.load_let(name, v)
    }

    #[inline]
    pub fn call_frame(&mut self, scope_index: usize) -> Result<(), VMError> {
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

    pub fn call_frame_memo(&mut self, scope_index: usize) -> Result<(), VMError> {
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

    // using this to distinguish VM runtime self vs rust self
    fn set_this(&mut self, mutable: bool) -> Result<(), VMError> {
        let this = self.next_value(format!("set self: mut {mutable}"));
        if mutable {
            self.frames.load_mut("self", this)
        } else {
            self.frames.load_let("self", this)
        }
    }

    /// Snapshots can't include modules or messages from in progress lifecycles
    pub fn snapshot(&self) -> Result<Vec<u8>, VMError> {
        let mut bytes = Vec::new();
        bytes.push(self.options.to_byte());
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
mod tests {
    use crate::builder::RigzBuilder;
    use crate::vm::VMOptions;
    use crate::{VMBuilder, Value, VM};

    #[test]
    fn options_snapshot() {
        let options = VMOptions {
            enable_logging: true,
            disable_modules: true,
            disable_variable_cleanup: true,
            ..Default::default()
        };
        let byte = options.to_byte();
        assert_eq!(VMOptions::from_byte(byte), options)
    }

    #[test]
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
