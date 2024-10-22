use crate::instructions::{Binary, BinaryAssign, Unary, UnaryAssign};
use crate::lifecycle::Lifecycle;
use crate::{
    generate_bin_op_methods, generate_builder, generate_unary_op_methods, BinaryOperation,
    CallFrame, Clear, Instruction, Module, Register, RigzType, Scope, UnaryOperation, VMError,
    Value, Variable,
};
use indexmap::map::Entry;
use indexmap::IndexMap;
use std::cell::RefCell;

use log::{trace, warn, Level};
use log_derive::logfn_inputs;
use nohash_hasher::BuildNoHashHasher;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

pub enum VMState {
    Running,
    Done(Value),
    Ran(Value),
}

impl VMState {
    #[inline]
    pub fn error(vm_error: VMError) -> Self {
        VMState::Done(vm_error.to_value())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMOptions {
    pub enable_logging: bool,
    pub disable_modules: bool,
    pub disable_variable_cleanup: bool,
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
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegisterValue {
    ScopeId(usize, Register),
    Register(Register),
    Value(Value),
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

#[derive(Debug, Clone)]
pub struct VM<'vm> {
    pub scopes: Vec<Scope<'vm>>,
    pub current: CallFrame<'vm>,
    pub frames: Vec<CallFrame<'vm>>,
    pub registers: IndexMap<usize, RefCell<RegisterValue>, BuildNoHashHasher<usize>>,
    pub stack: Vec<RegisterValue>,
    pub modules: IndexMap<&'static str, Box<dyn Module<'vm>>>,
    pub sp: usize,
    pub options: VMOptions,
    pub lifecycles: Vec<Lifecycle>,
}

impl<'vm> Default for VM<'vm> {
    #[inline]
    fn default() -> Self {
        Self {
            scopes: vec![Scope::new()],
            current: CallFrame::main(),
            frames: vec![],
            stack: vec![],
            registers: Default::default(),
            modules: Default::default(),
            sp: 0,
            options: Default::default(),
            lifecycles: Default::default(),
        }
    }
}

// todo all functions that return results should be mapped to Value::Error instead (when possible)
impl<'vm> VM<'vm> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    generate_builder!();

    #[inline]
    #[logfn_inputs(Trace)]
    pub fn insert_register(&mut self, register: Register, value: RegisterValue) {
        self.registers.insert(register, RefCell::new(value));
    }

    #[inline]
    #[logfn_inputs(Trace)]
    pub fn swap_register(&mut self, original: Register, reg: Register) {
        if original == reg {
            warn!("Called swap_register with same register {reg}");
            return;
        }

        let res = self
            .registers
            .insert(original, RefCell::new(RegisterValue::Register(reg)));
        match res {
            None => {
                panic!("Invalid call to swap_register {original} was not set");
            }
            Some(res) => {
                {
                    let b = res.borrow();
                    if let RegisterValue::Register(r) = b.deref() {
                        return self.swap_register(*r, reg);
                    }
                }
                self.registers.insert(reg, res);
            }
        }
    }

    #[inline]
    pub fn get_register(&self, register: Register) -> RegisterValue {
        match self.registers.get(&register) {
            None => VMError::EmptyRegister(format!("R{} is empty", register)).into(),
            Some(v) => v.borrow().clone(),
        }
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
        match self.get_register(register) {
            RegisterValue::ScopeId(scope, output) => self.handle_scope(scope, register, output),
            RegisterValue::Register(r) => self.resolve_register(r),
            RegisterValue::Value(v) => v,
        }
    }

    pub fn get_module_clone(&self, module: &'vm str) -> Result<Box<dyn Module<'vm>>, VMError> {
        match self.modules.get(&module) {
            None => Err(VMError::InvalidModule(module.to_string())),
            Some(m) => Ok(m.clone()),
        }
    }

    #[inline]
    pub fn update_register<F>(
        &mut self,
        register: Register,
        mut closure: F,
    ) -> Result<Option<Value>, VMError>
    where
        F: FnMut(&mut Value) -> Result<Option<Value>, VMError>,
    {
        let r = match self.registers.get(&register) {
            None => return Err(VMError::EmptyRegister(format!("R{} is empty", register))),
            Some(v) => {
                let mut v = v.borrow_mut();
                match v.deref_mut() {
                    RegisterValue::ScopeId(s, o) => {
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
    pub fn handle_scope(&mut self, scope: usize, original: Register, output: Register) -> Value {
        self.call_frame(scope, output);
        match self.run_scope() {
            VMState::Running => unreachable!(),
            VMState::Done(v) | VMState::Ran(v) => {
                self.insert_register(original, v.clone().into());
                v
            }
        }
    }

    /// Value is replaced with None, shifting the registers can break the program. Scopes are not evaluated, use `remove_register_eval_scope` instead.
    pub fn remove_register(&mut self, register: Register) -> RegisterValue {
        self.remove_register_value(register)
    }

    #[inline]
    fn remove_register_value(&mut self, register: Register) -> RegisterValue {
        match self.registers.get_mut(&register) {
            None => RegisterValue::Value(
                VMError::EmptyRegister(format!("R{} is empty", register)).to_value(),
            ),
            Some(v) => RefCell::replace(v, RegisterValue::Value(Value::None)),
        }
    }

    /// Value is replaced with None, shifting the registers breaks the program.

    pub fn remove_register_eval_scope(&mut self, register: Register) -> Value {
        match self.remove_register_value(register) {
            RegisterValue::ScopeId(scope, output) => self.handle_scope(scope, register, output),
            RegisterValue::Register(r) => self.remove_register_eval_scope(r),
            RegisterValue::Value(v) => v,
        }
    }

    pub fn process_ret(
        &mut self,
        output: Register,
        process: Option<fn(value: Value) -> VMState>,
    ) -> VMState {
        let current = self.current.output;
        let source = self.remove_register_eval_scope(current);
        self.insert_register(output, source.clone().into());
        match self.frames.pop() {
            None => return VMState::Done(source),
            Some(c) => {
                self.clear_frame(output);
                self.sp = c.scope_id;
                self.current = c;
                match process {
                    None => {}
                    Some(process) => return process(source),
                }
            }
        }
        VMState::Running
    }

    pub fn clear_frame(&mut self, output: Register) {
        if self.options.disable_variable_cleanup {
            return;
        }

        let variables = std::mem::take(&mut self.current.variables);
        for (_, var) in variables {
            match var {
                Variable::Let(r) | Variable::Mut(r) => {
                    if r == output {
                        continue;
                    }
                    self.remove_register(r);
                }
            };
        }
    }

    #[inline]
    pub fn process_instruction(&mut self, instruction: Instruction<'vm>) -> VMState {
        trace!("Running {:?}", instruction);
        self.current.pc += 1;
        match instruction {
            Instruction::Ret(output) => self.process_ret(output, None),
            instruction => self.process_core_instruction(instruction),
        }
    }

    pub fn process_instruction_scope(&mut self, instruction: Instruction<'vm>) -> VMState {
        trace!("Running {:?} (scope)", instruction);
        self.current.pc += 1;
        match instruction {
            Instruction::Ret(output) => self.process_ret(output, Some(VMState::Ran)),
            ins => self.process_core_instruction(ins),
        }
    }

    #[inline]
    /// scope_id must be valid when this is called, otherwise function will panic
    fn next_instruction(&self) -> Option<Instruction<'vm>> {
        let scope_id = self.sp;
        // TODO move &Scope to callframe
        let scope = &self.scopes[scope_id];
        scope.instructions.get(self.current.pc).cloned()
    }

    /// Generally this should be used instead of run. It will evaluate the VM & start lifecycles
    pub fn eval(&mut self) -> Result<Value, VMError> {
        match self.run() {
            Value::Error(e) => Err(e),
            v => Ok(v),
        }
    }

    pub fn run(&mut self) -> Value {
        loop {
            let instruction = match self.next_instruction() {
                // TODO this should probably be an error requiring explicit halt, result would be none
                None => return Value::None,
                Some(s) => s,
            };

            match self.process_instruction(instruction) {
                VMState::Ran(v) => {
                    return VMError::RuntimeError(format!("Unexpected ran state: {}", v)).to_value()
                }
                VMState::Running => {}
                VMState::Done(v) => return v,
            };
        }
    }

    fn run_scope(&mut self) -> VMState {
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
        match self.current.variables.entry(name) {
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
        match self.current.variables.entry(name) {
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
    pub fn call_frame(&mut self, scope_index: usize, output: Register) {
        if self.scopes.len() <= scope_index {
            self.insert_register(
                output,
                VMError::ScopeDoesNotExist(format!("{} does not exist", scope_index))
                    .to_value()
                    .into(),
            );
            return;
        }
        let current = std::mem::replace(
            &mut self.current,
            CallFrame::child(scope_index, self.frames.len(), output),
        );
        self.frames.push(current);
        self.sp = scope_index;
    }

    #[inline]
    pub fn call_frame_self(
        &mut self,
        scope_index: usize,
        output: Register,
        this: Register,
        mutable: bool,
    ) {
        self.call_frame(scope_index, output);
        let var = if mutable {
            Variable::Mut(this)
        } else {
            Variable::Let(this)
        };
        self.current.variables.insert("self", var);
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
    use crate::vm::VMOptions;
    use crate::{VMBuilder, Value, VM};

    #[test]
    fn options_snapshot() {
        let options = VMOptions {
            enable_logging: true,
            disable_modules: true,
            disable_variable_cleanup: true,
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
