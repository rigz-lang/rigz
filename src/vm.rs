use crate::instructions::{Binary, Unary};
use crate::{
    generate_bin_op_methods, generate_builder, generate_unary_op_methods, BinaryOperation,
    CallFrame, Instruction, Lifecycle, Module, Number, Register, RigzType, Scope,
    UnaryOperation, VMError, Value, Variable,
};
use indexmap::map::Entry;
use indexmap::IndexMap;
use log::{trace, Level};

pub enum VMState<'vm> {
    Running,
    Done(Value<'vm>),
    Ran(Value<'vm>),
}

#[derive(Clone, Debug, Default)]
pub struct VMOptions {
    pub enable_logging: bool,
    pub disable_modules: bool,
    pub disable_lifecyles: bool,
    pub disable_variable_cleanup: bool,
}

#[derive(Clone, Debug)]
pub struct VM<'vm> {
    pub scopes: Vec<Scope<'vm>>,
    pub current: CallFrame<'vm>,
    pub frames: Vec<CallFrame<'vm>>,
    pub registers: IndexMap<usize, Value<'vm>>,
    pub lifecycles: IndexMap<&'vm str, Lifecycle<'vm>>,
    pub modules: IndexMap<&'vm str, Module<'vm>>,
    pub sp: usize,
    pub options: VMOptions,
}

impl<'vm> Default for VM<'vm> {
    #[inline]
    fn default() -> Self {
        Self {
            scopes: vec![Scope::new()],
            current: Default::default(),
            frames: vec![],
            registers: Default::default(),
            lifecycles: Default::default(),
            modules: Default::default(),
            sp: 0,
            options: Default::default(),
        }
    }
}

impl<'vm> VM<'vm> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    generate_builder!();

    pub fn insert_register(&mut self, register: Register, value: Value<'vm>) {
        if register <= 1 {
            return;
        }

        self.registers.insert(register, value);
    }

    pub fn get_registers(&mut self, registers: Vec<Register>) -> Result<Vec<Value<'vm>>, VMError> {
        let len = registers.len();
        let mut result = Vec::with_capacity(len);
        for register in registers {
            result.push(self.get_register(register)?);
        }
        Ok(result)
    }

    #[inline]
    pub fn get_register(&mut self, register: Register) -> Result<Value<'vm>, VMError> {
        if register == 0 {
            return Ok(Value::None);
        }

        if register == 1 {
            return Ok(Value::Number(Number::Int(1)));
        }

        let v = match self.registers.get(&register) {
            None => return Err(VMError::EmptyRegister(format!("R{} is empty", register))),
            Some(v) => v.clone(),
        };

        if let Value::ScopeId(scope, output) = v {
            self.handle_scope(scope, register, output)
        } else {
            Ok(v)
        }
    }

    #[inline]
    pub fn get_register_mut(&'vm mut self, register: Register) -> Result<&'vm mut Value<'vm>, VMError> {
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
    ) -> Result<Value<'vm>, VMError> {
        self.call_frame(scope, output)?;
        match self.run_scope()? {
            VMState::Running => unreachable!(),
            VMState::Done(v) | VMState::Ran(v) => {
                self.insert_register(original, v.clone());
                Ok(v)
            }
        }
    }

    /// Value is replaced with None, shifting the registers breaks the program. Scopes are not evaluated, use `remove_register_eval_scope` instead.
    pub fn remove_register(&mut self, register: Register) -> Result<Value<'vm>, VMError> {
        self.remove_register_value(register)
    }

    #[inline]
    fn remove_register_value(&mut self, register: Register) -> Result<Value<'vm>, VMError> {
        match self.registers.get_mut(&register) {
            None => Err(VMError::EmptyRegister(format!("R{} is empty", register))),
            Some(v) => {
                let value = std::mem::take(v);
                *v = Value::None;
                Ok(value)
            }
        }
    }

    /// Value is replaced with None, shifting the registers breaks the program.

    pub fn remove_register_eval_scope(
        &mut self,
        register: Register,
    ) -> Result<Value<'vm>, VMError> {
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
        process: Option<fn(value: Value<'vm>) -> VMState<'vm>>,
    ) -> Result<VMState<'vm>, VMError> {
        let current = self.current.output;
        let source = self.get_register(current)?;
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

    pub fn process_instruction(
        &mut self,
        instruction: Instruction<'vm>,
    ) -> Result<VMState<'vm>, VMError> {
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
    ) -> Result<VMState<'vm>, VMError> {
        trace!("Running {:?} (scope)", instruction);
        self.current.pc += 1;
        match instruction {
            Instruction::Ret(output) => self.process_ret(output, Some(|s| VMState::Ran(s))),
            ins => self.process_core_instruction(ins),
        }
    }

    fn next_instruction(&self) -> Result<Option<Instruction<'vm>>, VMError> {
        let scope_id = self.sp;
        match self.scopes.get(scope_id) {
            None => Err(VMError::ScopeError(format!(
                "Scope {} does not exist",
                scope_id
            ))),
            Some(s) => match s.instructions.get(self.current.pc) {
                None => Ok(None),
                // TODO delay cloning until instruction is being used (some instructions can be copied with &)
                Some(s) => Ok(Some(s.clone())),
            },
        }
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

    pub fn run_scope(&mut self) -> Result<VMState<'vm>, VMError> {
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

    pub fn call_frame(&mut self, scope_index: usize, output: Register) -> Result<(), VMError> {
        if self.scopes.len() <= scope_index {
            return Err(VMError::ScopeDoesNotExist(format!(
                "{} does not exist",
                scope_index
            )));
        }
        let current = std::mem::take(&mut self.current);
        self.frames.push(current);
        self.sp = scope_index;
        self.current = CallFrame::child(scope_index, self.frames.len() - 1, output);
        Ok(())
    }

    pub fn run_lifecycles(&mut self) -> IndexMap<String, ()> {
        let mut futures = IndexMap::with_capacity(self.lifecycles.len());
        for (name, l) in &mut self.lifecycles {
            trace!("Starting Lifecycle: {}", name);
            futures.insert(name.to_string(), l.run());
        }
        futures
    }

    /// Snapshots can't include modules or messages from in progress lifecycles
    pub fn snapshot(&self) -> Vec<u8> {
        todo!()
    }

    pub fn load_snapshot(&mut self) {
        todo!()
    }
}
