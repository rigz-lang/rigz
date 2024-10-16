use crate::instructions::{Binary, Unary};
use crate::lifecycle::Lifecycle;
use crate::{
    generate_bin_op_methods, generate_builder, generate_unary_op_methods, BinaryOperation,
    CallFrame, Instruction, Module, Number, Register, RigzType, Scope, UnaryOperation, VMError,
    Value, Variable,
};
use indexmap::map::Entry;
use indexmap::IndexMap;
use log::{trace, warn, Level};
use nohash_hasher::BuildNoHashHasher;
use std::fmt::{Debug, Formatter};

pub enum VMState {
    Running,
    Done(Value),
    Ran(Value),
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

pub struct VM<'vm> {
    pub scopes: Vec<Scope<'vm>>,
    pub current: CallFrame<'vm>,
    pub frames: Vec<CallFrame<'vm>>,
    pub registers: IndexMap<usize, Value, BuildNoHashHasher<usize>>,
    pub stack: Vec<Value>,
    pub modules: IndexMap<&'vm str, Box<dyn Module<'vm>>>,
    pub sp: usize,
    pub options: VMOptions,
    pub lifecycles: Vec<Lifecycle>,
}

impl<'vm> Debug for VM<'vm> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "VM(current={:?},scopes={:?},frames={:?},registers={:?},stack={:?},modules={:?},sp={},options={:?},lifecycles={:?})",
               self.current,
               self.scopes,
               self.frames,
               self.registers,
               self.stack,
               self.modules.keys(),
               self.sp,
               self.options,
               self.lifecycles
        )
    }
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
    pub fn insert_register(&mut self, register: Register, value: Value) {
        match register {
            0 | 1 => {
                warn!("Insert Register called for {}, value not saved", register)
            }
            register => {
                self.registers.insert(register, value);
            }
        }
    }

    #[inline]
    pub fn get_register(&mut self, register: Register) -> Result<Value, VMError> {
        match register {
            0 => Ok(Value::None),
            1 => Ok(Value::Number(Number::one())),
            register => match self.registers.get(&register) {
                None => Err(VMError::EmptyRegister(format!("R{} is empty", register))),
                Some(v) => Ok(v.clone()),
            },
        }
    }

    pub fn resolve_registers(&mut self, registers: Vec<Register>) -> Result<Vec<Value>, VMError> {
        let len = registers.len();
        let mut result = Vec::with_capacity(len);
        for register in registers {
            result.push(self.resolve_register(register)?);
        }
        Ok(result)
    }

    #[inline]
    pub fn resolve_register(&mut self, register: Register) -> Result<Value, VMError> {
        let v = self.get_register(register)?;

        if let Value::ScopeId(scope, output) = v {
            self.handle_scope(scope, register, output)
        } else {
            Ok(v)
        }
    }

    pub fn get_module_clone(&self, module: &'vm str) -> Result<Box<dyn Module<'vm>>, VMError> {
        match self.modules.get(&module) {
            None => Err(VMError::InvalidModule(module.to_string())),
            Some(m) => Ok(m.clone()),
        }
    }

    #[inline]
    pub fn get_register_mut(&mut self, register: Register) -> Result<&mut Value, VMError> {
        match self.registers.get_mut(&register) {
            None => Err(VMError::EmptyRegister(format!("R{} is empty", register))),
            Some(v) => Ok(v),
        }
    }

    #[inline]
    pub fn handle_scope(
        &mut self,
        scope: usize,
        original: Register,
        output: Register,
    ) -> Result<Value, VMError> {
        self.call_frame(scope, output)?;
        match self.run_scope()? {
            VMState::Running => unreachable!(),
            VMState::Done(v) | VMState::Ran(v) => {
                self.insert_register(original, v.clone());
                Ok(v)
            }
        }
    }

    /// Value is replaced with None, shifting the registers can break the program. Scopes are not evaluated, use `remove_register_eval_scope` instead.
    pub fn remove_register(&mut self, register: Register) -> Result<Value, VMError> {
        match register {
            0 => Ok(Value::None),
            1 => Ok(Value::Number(Number::one())),
            register => self.remove_register_value(register),
        }
    }

    #[inline]
    fn remove_register_value(&mut self, register: Register) -> Result<Value, VMError> {
        match self.registers.get_mut(&register) {
            None => Err(VMError::EmptyRegister(format!("R{} is empty", register))),
            Some(v) => {
                let value = std::mem::take(v);
                Ok(value)
            }
        }
    }

    /// Value is replaced with None, shifting the registers breaks the program.

    pub fn remove_register_eval_scope(&mut self, register: Register) -> Result<Value, VMError> {
        let value = self.remove_register_value(register)?;

        if let Value::ScopeId(scope, output) = value {
            self.handle_scope(scope, register, output)
        } else {
            Ok(value)
        }
    }

    pub fn process_ret(
        &mut self,
        output: Register,
        process: Option<fn(value: Value) -> VMState>,
    ) -> Result<VMState, VMError> {
        let current = self.current.output;
        let source = self.resolve_register(current)?;
        self.insert_register(output, source.clone());
        match self.frames.pop() {
            None => return Ok(VMState::Done(source)),
            Some(c) => {
                self.clear_frame()?;
                self.sp = c.scope_id;
                self.current = c;
                match process {
                    None => {}
                    Some(process) => return Ok(process(source)),
                }
            }
        }
        Ok(VMState::Running)
    }

    pub fn clear_frame(&mut self) -> Result<(), VMError> {
        if self.options.disable_variable_cleanup {
            return Ok(());
        }

        let variables = std::mem::take(&mut self.current.variables);
        for reg in variables.values() {
            let _ = match reg {
                Variable::Let(r) | Variable::Mut(r) => self.remove_register(*r)?,
            };
        }
        Ok(())
    }

    #[inline]
    pub fn process_instruction(
        &mut self,
        instruction: Instruction<'vm>,
    ) -> Result<VMState, VMError> {
        trace!("Running {:?}", instruction);
        self.current.pc += 1;
        match instruction {
            Instruction::Ret(output) => self.process_ret(output, None),
            instruction => self.process_core_instruction(instruction),
        }
    }

    pub fn process_instruction_scope(
        &mut self,
        instruction: Instruction<'vm>,
    ) -> Result<VMState, VMError> {
        trace!("Running {:?} (scope)", instruction);
        self.current.pc += 1;
        match instruction {
            Instruction::Ret(output) => self.process_ret(output, Some(VMState::Ran)),
            ins => self.process_core_instruction(ins),
        }
    }

    #[inline]
    /// scope_id must be valid when this is called, otherwise function will panic
    fn next_instruction(&self) -> Result<Option<Instruction<'vm>>, VMError> {
        let scope_id = self.sp;
        // TODO move &Scope to callframe
        let scope = &self.scopes[scope_id];
        match scope.instructions.get(self.current.pc) {
            None => Ok(None),
            // TODO delay cloning until instruction is being used (some instructions can be copied with &)
            Some(s) => Ok(Some(s.clone())),
        }
    }

    /// Generally this should be used instead of run. It will evaluate the VM & start lifecycles
    pub fn eval(&mut self) -> Result<Value, VMError> {
        self.run()
    }

    pub fn run(&mut self) -> Result<Value, VMError> {
        loop {
            let instruction = match self.next_instruction()? {
                // TODO this should probably be an error requiring explicit halt, result would be none
                None => return Ok(Value::None),
                Some(s) => s,
            };

            match self.process_instruction(instruction)? {
                VMState::Ran(v) => {
                    return Err(VMError::RuntimeError(format!(
                        "Unexpected ran state: {}",
                        v
                    )))
                }
                VMState::Running => {}
                VMState::Done(v) => return Ok(v),
            };
        }
    }

    fn run_scope(&mut self) -> Result<VMState, VMError> {
        loop {
            let instruction = match self.next_instruction()? {
                // TODO this should probably be an error requiring explicit halt, result would be none
                None => return Ok(VMState::Done(Value::None)),
                Some(s) => s,
            };

            match self.process_instruction_scope(instruction)? {
                VMState::Running => {}
                s => return Ok(s),
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
    pub fn call_frame(&mut self, scope_index: usize, output: Register) -> Result<(), VMError> {
        if self.scopes.len() <= scope_index {
            return Err(VMError::ScopeDoesNotExist(format!(
                "{} does not exist",
                scope_index
            )));
        }
        let current = std::mem::replace(
            &mut self.current,
            CallFrame::child(scope_index, self.frames.len(), output),
        );
        self.frames.push(current);
        self.sp = scope_index;
        Ok(())
    }

    /// Snapshots can't include modules or messages from in progress lifecycles
    pub fn snapshot(&self) -> Result<Vec<u8>, VMError> {
        let mut bytes = Vec::new();
        bytes.push(self.options.to_byte());
        bytes.extend((self.sp as u64).to_le_bytes());
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
        builder.add_load_instruction(1, Value::Bool(true));
        let vm = builder.build();
        let bytes = vm.snapshot().expect("snapshot failed");
        let mut vm2 = VM::default();
        vm2.load_snapshot(bytes).expect("load failed");
        assert_eq!(vm2.options, vm.options);
        assert_eq!(vm2.sp, vm.sp);
    }
}
