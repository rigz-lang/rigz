use crate::instructions::{Binary, Unary};
use crate::{
    generate_bin_op_methods, generate_builder, generate_unary_op_methods, BinaryOperation,
    CallFrame, Instruction, Lifecycle, Logical, Module, Number, Register, Reverse, RigzType, Scope,
    UnaryOperation, VMError, Value, Variable,
};
use indexmap::map::Entry;
use indexmap::IndexMap;
use log::{error, log, trace, Level};

pub enum VMState<'vm> {
    Running,
    Done(Value<'vm>),
    Ran(Value<'vm>),
}

#[derive(Clone, Debug, Default)]
pub struct VMOptions {
    enable_logging: bool,
    disable_modules: bool,
}

#[derive(Clone, Debug)]
pub struct VM<'vm> {
    pub scopes: Vec<Scope<'vm>>,
    pub current: CallFrame<'vm>,
    pub frames: Vec<CallFrame<'vm>>,
    pub registers: IndexMap<usize, Value<'vm>>,
    pub lifecycles: Vec<Lifecycle<'vm>>,
    pub modules: IndexMap<&'vm str, Module<'vm>>,
    pub sp: usize,
    pub options: VMOptions,
}

impl<'vm> VM<'vm> {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            current: Default::default(),
            frames: vec![],
            registers: Default::default(),
            lifecycles: vec![],
            modules: Default::default(),
            sp: 0,
            options: Default::default(),
        }
    }

    generate_builder!();

    pub fn insert_register(&mut self, register: Register, value: Value<'vm>) {
        if register <= 1 {
            return;
        }

        self.registers.insert(register, value);
    }

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
            self.call_frame(scope, output)?;
            match self.run_scope()? {
                VMState::Running => unreachable!(),
                VMState::Done(v) | VMState::Ran(v) => {
                    self.insert_register(register, v.clone());
                    Ok(v)
                }
            }
        } else {
            Ok(v)
        }
    }

    fn handle_binary(&mut self, binary: Binary) -> Result<(), VMError> {
        let Binary {
            op,
            lhs,
            rhs,
            output,
        } = binary;
        let lhs = self.get_register(lhs)?;
        let rhs = self.get_register(rhs)?;
        let v = match op {
            BinaryOperation::Add => lhs + rhs,
            BinaryOperation::Sub => lhs - rhs,
            BinaryOperation::Shr => lhs >> rhs,
            BinaryOperation::Shl => lhs << rhs,
            BinaryOperation::Eq => Value::Bool(lhs == rhs),
            BinaryOperation::Neq => Value::Bool(lhs != rhs),
            BinaryOperation::Mul => lhs * rhs,
            BinaryOperation::Div => lhs / rhs,
            BinaryOperation::Rem => lhs % rhs,
            BinaryOperation::BitOr => lhs | rhs,
            BinaryOperation::BitAnd => lhs & rhs,
            BinaryOperation::BitXor => lhs ^ rhs,
            BinaryOperation::And => lhs.and(rhs),
            BinaryOperation::Or => lhs.or(rhs),
            BinaryOperation::Xor => lhs.xor(rhs),
            BinaryOperation::Gt => Value::Bool(lhs > rhs),
            BinaryOperation::Gte => Value::Bool(lhs >= rhs),
            BinaryOperation::Lt => Value::Bool(lhs < rhs),
            BinaryOperation::Lte => Value::Bool(lhs <= rhs),
        };

        self.insert_register(output, v);
        Ok(())
    }

    fn handle_unary(&mut self, unary: Unary) -> Result<(), VMError> {
        let Unary { op, from, output } = unary;
        let val = self.get_register(from)?;
        match op {
            UnaryOperation::Neg => {
                self.insert_register(output, -val);
            }
            UnaryOperation::Not => {
                self.insert_register(output, !val);
            }
            UnaryOperation::PrintLn => {
                println!("{}", val);
                self.insert_register(output, val);
            }
            UnaryOperation::EPrintLn => {
                eprintln!("{}", val);
                self.insert_register(output, val);
            }
            UnaryOperation::Print => {
                print!("{}", val);
                self.insert_register(output, val);
            }
            UnaryOperation::EPrint => {
                eprint!("{}", val);
                self.insert_register(output, val);
            }
            UnaryOperation::Reverse => {
                self.insert_register(output, val.reverse());
            }
        }
        Ok(())
    }

    pub fn process_core_instruction(
        &mut self,
        instruction: Instruction<'vm>,
    ) -> Result<VMState<'vm>, VMError> {
        match instruction {
            Instruction::Halt(r) => return Ok(VMState::Done(self.get_register(r)?)),
            Instruction::Unary(u) => self.handle_unary(u)?,
            Instruction::Binary(b) => self.handle_binary(b)?,
            Instruction::Load(r, v) => self.insert_register(r, v),
            Instruction::LoadLetRegister(name, register) => self.load_let(name, register)?,
            Instruction::LoadMutRegister(name, register) => self.load_mut(name, register)?,
            Instruction::Copy(from, to) => {
                let copy = self.get_register(from)?;
                self.insert_register(to, copy);
            }
            Instruction::Cast {
                from,
                rigz_type,
                to,
            } => {
                let value = self.get_register(from)?;
                self.insert_register(to, value.cast(rigz_type)?);
            }
            Instruction::CallEq(a, b, scope_index, output) => {
                let a = self.get_register(a)?;
                let b = self.get_register(b)?;
                if a == b {
                    self.call_frame(scope_index, output)?;
                }
            }
            Instruction::CallNeq(a, b, scope_index, output) => {
                let a = self.get_register(a)?;
                let b = self.get_register(b)?;
                if a != b {
                    self.call_frame(scope_index, output)?;
                }
            }
            Instruction::IfElse {
                truthy,
                if_scope,
                else_scope,
                output,
            } => {
                if self.get_register(truthy)?.to_bool() {
                    self.call_frame(if_scope, output)?;
                } else {
                    self.call_frame(else_scope, output)?;
                }
            }
            Instruction::GetVariable(name, reg) => match self.current.get_variable(&name, self) {
                None => {
                    return Err(VMError::VariableDoesNotExist(format!(
                        "Variable {} does not exist",
                        name
                    )))
                }
                Some(s) => match self.registers.get(&s) {
                    None => {
                        return Err(VMError::EmptyRegister(format!(
                            "Register {} does not exist",
                            s
                        )))
                    }
                    Some(v) => self.insert_register(reg, v.clone()),
                },
            },
            Instruction::CallModule {
                module,
                function,
                args,
                output,
            } => {
                if self.options.disable_modules {
                    error!("Modules are disabled");
                    self.insert_register(
                        output,
                        VMError::UnsupportedOperation(format!(
                            "Modules are disabled: Failed to call module {}.{}",
                            module, function
                        ))
                        .to_value(),
                    );
                    return Ok(VMState::Running);
                }
                let f = match self.modules.get(module) {
                    None => {
                        return Err(VMError::InvalidModule(format!(
                            "Module {} does not exist",
                            module
                        )))
                    }
                    Some(m) => match m.functions.get(function) {
                        None => {
                            return Err(VMError::InvalidModuleFunction(format!(
                                "Module {}.{} does not exist",
                                module, function
                            )))
                        }
                        Some(f) => f.clone(),
                    },
                };
                let mut inner_args = Vec::with_capacity(args.len());
                for arg in args {
                    inner_args.push(self.get_register(arg)?);
                }
                let v = f(inner_args);
                self.insert_register(output, v)
            }
            Instruction::CallExtensionModule {
                module,
                function,
                this,
                args,
                output,
            } => {
                if self.options.disable_modules {
                    error!("Modules are disabled");
                    self.insert_register(
                        output,
                        VMError::UnsupportedOperation(format!(
                            "Modules are disabled: Failed to call extension {}::{}.{}",
                            module, this, function
                        ))
                        .to_value(),
                    );
                    return Ok(VMState::Running);
                }
                let m = match self.modules.get(module) {
                    None => {
                        return Err(VMError::InvalidModule(format!(
                            "Module {} does not exist",
                            module
                        )))
                    }
                    Some(m) => m.clone(),
                };
                let this = self.get_register(this)?;
                let rigz_type = this.rigz_type();
                let f = match m.extension_functions.get(&rigz_type) {
                    None => match m.extension_functions.get(&RigzType::Any) {
                        None => {
                            return Err(VMError::InvalidModuleFunction(format!(
                                "Module {}.{:?} does not exist (Any does not exist)",
                                module, rigz_type
                            )))
                        }
                        Some(def) => match def.get(function) {
                            None => {
                                return Err(VMError::InvalidModuleFunction(format!(
                                    "Module extension {}.{} does not exist",
                                    module, function
                                )))
                            }
                            Some(f) => f.clone(),
                        },
                    },
                    Some(def) => match def.get(function) {
                        None => {
                            return Err(VMError::InvalidModuleFunction(format!(
                                "Module extension {}.{} does not exist",
                                module, function
                            )))
                        }
                        Some(f) => f.clone(),
                    },
                };
                let mut inner_args = Vec::with_capacity(args.len());
                for arg in args {
                    inner_args.push(self.get_register(arg)?);
                }
                let v = f(this, inner_args);
                self.insert_register(output, v)
            }
            Instruction::Log(level, tmpl, args) => {
                if !self.options.enable_logging {
                    return Ok(VMState::Running);
                }

                let mut res = (*tmpl).to_string();
                for arg in args {
                    let l = arg.to_string();
                    res = res.replacen("{}", l.as_str(), 1);
                }
                log!(level, "{}", res)
            }
            Instruction::Puts(args) => println!(
                "{}",
                args.iter()
                    .map(|v| v.to_string())
                    .fold(String::new(), |mut a, b| {
                        a.push_str(", ");
                        a.push_str(b.as_str());
                        a
                    })
            ),
            Instruction::Call(scope_index, register) => self.call_frame(scope_index, register)?,
            Instruction::Ret(r) => {
                return Err(VMError::UnsupportedOperation(format!(
                    "Ret not handled by parent function: R{}",
                    r
                )))
            }
        };
        Ok(VMState::Running)
    }

    pub fn process_instruction(
        &mut self,
        instruction: Instruction<'vm>,
    ) -> Result<VMState<'vm>, VMError> {
        trace!("Running {:?}", instruction);
        self.current.pc += 1;
        match instruction {
            Instruction::Ret(output) => {
                let current = self.current.output;
                let source = self.get_register(current)?;
                self.insert_register(output, source.clone());
                match self.frames.pop() {
                    None => return Ok(VMState::Done(source)),
                    Some(c) => {
                        self.clear_frame()?;
                        self.current = c;
                    }
                }
            }
            instruction => return self.process_core_instruction(instruction),
        };
        Ok(VMState::Running)
    }

    pub(crate) fn clear_frame(&mut self) -> Result<(), VMError> {
        let variables = std::mem::take(&mut self.current.variables);
        for reg in variables.values() {
            let _ = match reg {
                Variable::Let(r) => self.get_register(*r)?,
                Variable::Mut(r) => self.get_register(*r)?,
            };
        }
        Ok(())
    }

    pub fn process_instruction_scope(
        &mut self,
        instruction: Instruction<'vm>,
    ) -> Result<VMState<'vm>, VMError> {
        trace!("Running {:?} (scope)", instruction);
        self.current.pc += 1;
        match instruction {
            Instruction::Call(scope_index, register) => self.call_frame(scope_index, register)?,
            Instruction::Ret(output) => {
                let current = self.current.output;
                let source = self.get_register(current)?;
                self.insert_register(output, source.clone());
                return match self.frames.pop() {
                    None => Ok(VMState::Done(source)),
                    Some(c) => {
                        self.clear_frame()?;
                        self.current = c;
                        Ok(VMState::Ran(source))
                    }
                };
            }
            ins => return self.process_core_instruction(ins),
        };
        Ok(VMState::Running)
    }

    fn next_instruction(&self) -> Result<Option<Instruction<'vm>>, VMError> {
        let scope_id = self.current.scope_id;
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
        self.current = CallFrame::child(scope_index, self.frames.len() - 1, output);
        Ok(())
    }
}

impl<'vm> Default for VM<'vm> {
    fn default() -> Self {
        VM::new()
    }
}
