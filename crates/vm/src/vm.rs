use crate::lifecycle::{Lifecycle, TestResults};
use crate::{generate_builder, CallFrame, Instruction, Scope, Variable};
use crate::{out, outln, Module, Register, RigzBuilder, VMError, Value};
use indexmap::map::Entry;
use indexmap::IndexMap;
use itertools::Itertools;
use log_derive::logfn_inputs;
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::DerefMut;
use std::time::{Duration, Instant};

pub enum VMState {
    Running,
    Done(Value),
    Ran(Value),
}

impl VMState {
    #[inline]
    pub fn error(vm_error: VMError) -> Self {
        VMState::Done(vm_error.into())
    }
}

impl From<VMError> for VMState {
    #[inline]
    fn from(value: VMError) -> Self {
        VMState::error(value)
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
pub enum RegisterValue {
    ScopeId(usize, Register, Vec<Register>),
    Register(Register),
    Value(Value),
    Constant(usize),
}

impl RegisterValue {
    pub fn resolve(self, vm: &mut VM, register: Register) -> Value {
        match self {
            RegisterValue::ScopeId(scope, output, args) => {
                vm.handle_scope(scope, register, args, output)
            }
            RegisterValue::Register(r) => vm.resolve_register(r),
            RegisterValue::Value(v) => v,
            RegisterValue::Constant(c) => vm.get_constant(c),
        }
    }
}

impl From<usize> for RegisterValue {
    fn from(value: usize) -> Self {
        RegisterValue::Register(value)
    }
}

impl<T: Into<Value>> From<T> for RegisterValue {
    #[inline]
    fn from(value: T) -> Self {
        RegisterValue::Value(value.into())
    }
}

#[derive(Clone, Debug)]
pub struct VM<'vm> {
    pub scopes: Vec<Scope<'vm>>,
    pub current: RefCell<CallFrame<'vm>>,
    pub frames: Vec<RefCell<CallFrame<'vm>>>,
    pub modules: IndexMap<&'static str, Box<dyn Module<'vm>>>,
    pub stack: Vec<Value>,
    pub sp: usize,
    pub options: VMOptions,
    pub lifecycles: Vec<Lifecycle>,
    pub constants: Vec<Value>,
}

impl<'vm> RigzBuilder<'vm> for VM<'vm> {
    generate_builder!();

    fn build(self) -> VM<'vm> {
        self
    }
}

impl<'vm> Default for VM<'vm> {
    #[inline]
    fn default() -> Self {
        Self {
            scopes: vec![Scope::default()],
            current: RefCell::new(CallFrame::main()),
            frames: vec![],
            modules: Default::default(),
            sp: 0,
            options: Default::default(),
            lifecycles: Default::default(),
            constants: Default::default(),
            stack: Default::default(),
        }
    }
}

// todo all functions that return results should be mapped to Value::Error instead (when possible)
impl<'vm> VM<'vm> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    #[logfn_inputs(Trace, fmt = "insert_register(vm={:#p} register={}, value={:?})")]
    pub fn insert_register(
        &self,
        register: Register,
        value: RegisterValue,
    ) -> Option<RefCell<RegisterValue>> {
        match value {
            RegisterValue::Register(dest) if dest == register => None,
            _ => self
                .current
                .borrow_mut()
                .registers
                .insert(register, RefCell::new(value)),
        }
    }

    #[inline]
    #[logfn_inputs(Trace, fmt = "swap_register(vm={:#p} original={}, dest={})")]
    pub fn swap_register(&self, original: Register, dest: Register) {
        self.current
            .borrow_mut()
            .swap_register(original, dest, self)
    }

    #[inline]
    pub fn get_register(&self, register: Register) -> RegisterValue {
        self.current.borrow().get_register(register, self)
    }

    pub fn resolve_registers(&mut self, registers: Vec<Register>) -> Vec<Value> {
        let len = registers.len();
        let mut result = Vec::with_capacity(len);
        for register in registers {
            result.push(self.resolve_register(register));
        }
        result
    }

    #[inline]
    pub fn resolve_register(&mut self, register: Register) -> Value {
        let v = self.get_register(register);
        v.resolve(self, register)
    }

    pub fn get_module_clone(&self, module: &'vm str) -> Result<Box<dyn Module<'vm>>, VMError> {
        match self.modules.get(&module) {
            None => Err(VMError::InvalidModule(module.to_string())),
            Some(m) => Ok(m.clone()),
        }
    }

    #[inline]
    pub fn update_register<F>(
        &self,
        register: Register,
        mut closure: F,
    ) -> Result<Option<Value>, VMError>
    where
        F: FnMut(&mut Value) -> Result<Option<Value>, VMError>,
    {
        let r = match self.current.borrow_mut().registers.get(&register) {
            None => return Err(VMError::EmptyRegister(format!("R{} is empty", register))),
            Some(v) => {
                let mut v = v.borrow_mut();
                match v.deref_mut() {
                    RegisterValue::Constant(c) => {
                        return Err(VMError::UnsupportedOperation(format!(
                            "Constants cannot be mutated {c}"
                        )))
                    }
                    RegisterValue::ScopeId(s, o, _args) => {
                        return Err(VMError::UnsupportedOperation(format!(
                            "Scopes are not implemented yet - Scope {s} R{o}"
                        )))
                    }
                    RegisterValue::Register(r) => *r,
                    RegisterValue::Value(v) => return closure(v),
                }
            }
        };
        self.update_register(r, closure)
    }

    // todo create update_registers to support multiple mutable values at the same time

    #[inline]
    pub fn handle_scope(
        &mut self,
        scope: usize,
        original: Register,
        args: Vec<Register>,
        output: Register,
    ) -> Value {
        let current = self.sp;
        match self.call_frame(scope, args, output) {
            Ok(_) => {}
            Err(e) => return e.into(),
        };
        let mut v = match self.run_scope() {
            VMState::Running => unreachable!(),
            VMState::Done(v) | VMState::Ran(v) => v,
        };
        while current != self.sp {
            v = match self.run_scope() {
                VMState::Running => unreachable!(),
                VMState::Done(v) | VMState::Ran(v) => v,
            };
        }
        self.insert_register(output, v.clone().into());
        v
    }

    /// Value is replaced with None, shifting the registers can break the program. Scopes are not evaluated, use `remove_register_eval_scope` instead.
    pub fn remove_register(&self, register: Register) -> RegisterValue {
        self.current.borrow_mut().remove_register(register, self)
    }

    /// Value is replaced with None, shifting the registers breaks the program.

    pub fn remove_register_eval_scope(&mut self, register: Register) -> Value {
        let rv = self.remove_register(register);
        rv.resolve(self, register)
    }

    pub fn get_constant(&self, index: usize) -> Value {
        match self.constants.get(index) {
            None => VMError::RuntimeError(format!("Constant {index} does not exist")).into(),
            Some(v) => v.clone(),
        }
    }

    pub fn process_ret(
        &mut self,
        output: Register,
        process: Option<fn(value: Value) -> VMState>,
    ) -> VMState {
        let current = self.current.borrow().output;
        let source = self.resolve_register(current);
        match self.frames.pop() {
            None => return VMState::Done(source),
            Some(c) => {
                self.sp = c.borrow().scope_id;
                self.current = c;
                self.insert_register(output, source.clone().into());
                match process {
                    None => {}
                    Some(process) => return process(source),
                }
            }
        }
        VMState::Running
    }

    #[inline]
    pub fn process_instruction(&mut self, instruction: Instruction<'vm>) -> VMState {
        self.current.borrow_mut().pc += 1;
        match instruction {
            Instruction::Ret(output) => self.process_ret(output, None),
            instruction => self.process_core_instruction(instruction),
        }
    }

    pub fn process_instruction_scope(&mut self, instruction: Instruction<'vm>) -> VMState {
        self.current.borrow_mut().pc += 1;
        match instruction {
            Instruction::Ret(output) => self.process_ret(output, Some(VMState::Ran)),
            ins => self.process_core_instruction(ins),
        }
    }

    #[inline]
    #[log_derive::logfn(Debug)]
    fn next_instruction(&self) -> Option<Instruction<'vm>> {
        let scope_id = self.sp;
        // TODO move &Scope to callframe
        // scope_id must be valid when this is called, otherwise function will panic
        let scope = &self.scopes[scope_id];
        scope.instructions.get(self.current.borrow().pc).cloned()
    }

    /// Generally this should be used instead of run. It will evaluate the VM & start lifecycles
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

    pub fn run(&mut self) -> Value {
        loop {
            match self.step() {
                None => {}
                Some(v) => return v,
            }
        }
    }

    #[inline]
    fn step(&mut self) -> Option<Value> {
        let instruction = match self.next_instruction() {
            // TODO this should probably be an error requiring explicit halt, result would be none
            None => return Some(Value::None),
            Some(s) => s,
        };

        match self.process_instruction(instruction) {
            VMState::Ran(v) => {
                return Some(VMError::RuntimeError(format!("Unexpected ran state: {}", v)).into())
            }
            VMState::Running => {}
            VMState::Done(v) => return Some(v),
        };
        None
    }

    pub fn run_within(&mut self, duration: Duration) -> Value {
        let now = Instant::now();
        let end = now + duration;
        loop {
            let current = Instant::now();
            if current > end {
                return Value::Error(VMError::TimeoutError(format!(
                    "Exceeded runtime {duration:?} - {:?}",
                    end - current
                )));
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
                    let Instruction::Ret(o) =
                        s.instructions.last().expect("No instructions for scope")
                    else {
                        unreachable!("Invalid Scope")
                    };
                    Some((index, *o, s.named))
                }
                Some(_) => None,
            })
            .collect();

        let mut passed = 0;
        let mut failed = 0;
        let start = Instant::now();
        let mut failure_messages = Vec::new();
        for (s, r, named) in test_scopes {
            out!("test {named} ... ");
            self.sp = s;
            self.current = RefCell::new(CallFrame {
                scope_id: s,
                pc: 0,
                registers: Default::default(),
                variables: Default::default(),
                parent: None,
                output: r,
            });
            let v = self.run();
            match v {
                Value::Error(e) => {
                    outln!("\x1b[31mFAILED\x1b[0m");
                    failed += 1;
                    failure_messages.push((named.to_string(), e));
                }
                _ => {
                    outln!("\x1b[32mok\x1b[0m");
                    passed += 1;
                }
            };
        }

        TestResults {
            passed,
            failed,
            failure_messages,
            duration: Instant::now() - start,
        }
    }

    pub fn run_scope(&mut self) -> VMState {
        loop {
            let instruction = match self.next_instruction() {
                // TODO this should probably be an error requiring explicit halt, result would be none
                None => return VMState::Done(Value::None),
                Some(s) => s,
            };

            match self.process_instruction_scope(instruction) {
                VMState::Running => {}
                s => return s,
            };
        }
    }

    pub fn load_mut(&mut self, name: &'vm str, reg: Register) -> Result<(), VMError> {
        match self.current.borrow_mut().variables.entry(name) {
            Entry::Occupied(mut var) => match var.get() {
                Variable::Let(_) => {
                    return Err(VMError::UnsupportedOperation(format!(
                        "Cannot overwrite let variable: {}",
                        *var.key()
                    )))
                }
                Variable::Mut(_) => {
                    var.insert(Variable::Mut(reg));
                }
            },
            Entry::Vacant(e) => {
                e.insert(Variable::Mut(reg));
            }
        }
        Ok(())
    }

    pub fn load_let(&mut self, name: &'vm str, reg: Register) -> Result<(), VMError> {
        match self.current.borrow_mut().variables.entry(name) {
            Entry::Occupied(v) => {
                return Err(VMError::UnsupportedOperation(format!(
                    "Cannot overwrite let variable: {}",
                    *v.key()
                )))
            }
            Entry::Vacant(e) => {
                e.insert(Variable::Let(reg));
            }
        }
        Ok(())
    }

    #[inline]
    pub fn call_frame(
        &mut self,
        scope_index: usize,
        args: Vec<Register>,
        output: Register,
    ) -> Result<(), VMError> {
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

        let current = std::mem::replace(
            &mut self.current,
            RefCell::new(CallFrame::child(scope_index, self.frames.len(), output)),
        );

        self.frames.push(current);
        self.sp = scope_index;
        let args = self.scopes[scope_index].args.clone().into_iter().zip(args);
        for ((arg, mutable), reg) in args {
            if mutable {
                self.load_mut(arg, reg)?;
            } else {
                self.load_let(arg, reg)?;
            }
        }
        Ok(())
    }

    pub fn call_frame_memo(
        &mut self,
        scope_index: usize,
        args: Vec<Register>,
        output: Register,
    ) -> Result<(), VMError> {
        let call_args = self.resolve_registers(args.clone());
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
                let value = self.handle_scope(scope_index, output, args, output);
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

                        memo.results.insert(call_args, value.clone());
                        value
                    }
                }
            }
            Some(s) => s,
        };
        self.insert_register(output, value.into());
        Ok(())
    }

    #[inline]
    pub fn call_frame_self(
        &mut self,
        scope_index: usize,
        this: Register,
        output: Register,
        args: Vec<Register>,
        mutable: bool,
    ) -> Result<(), VMError> {
        self.call_frame(scope_index, args, output)?;
        let var = if mutable {
            Variable::Mut(this)
        } else {
            Variable::Let(this)
        };
        self.current.borrow_mut().variables.insert("self", var);
        Ok(())
    }

    #[inline]
    pub fn call_frame_self_memo(
        &mut self,
        scope_index: usize,
        this: Register,
        output: Register,
        args: Vec<Register>,
        mutable: bool,
    ) -> Result<(), VMError> {
        todo!()
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
        self.options = VMOptions::from_byte(bytes[0]);
        let mut sp = [0; 8];
        for (i, b) in bytes[1..9].iter().enumerate() {
            sp[i] = *b;
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
        builder.add_load_instruction(1, Value::Bool(true).into());
        let vm = builder.build();
        let bytes = vm.snapshot().expect("snapshot failed");
        let mut vm2 = VM::default();
        vm2.load_snapshot(bytes).expect("load failed");
        assert_eq!(vm2.options, vm.options);
        assert_eq!(vm2.sp, vm.sp);
        // assert_eq!(vm2.get_register(1), Value::Bool(true).into());
    }
}
