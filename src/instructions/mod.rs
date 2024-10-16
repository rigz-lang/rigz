mod binary;
mod unary;

pub use binary::{Binary, BinaryOperation};
use log::{log, Level};
pub use unary::{Unary, UnaryOperation};

use crate::vm::VMState;
use crate::{Register, RigzType, VMError, Value, VM};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Clear {
    One(Register),
    Two(Register, Register),
    Three(Register, Register, Register),
    Many(Vec<Register>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction<'vm> {
    Halt(Register),
    Unary(Unary),
    Binary(Binary),
    UnaryAssign(Unary),
    BinaryAssign(Binary),
    Load(Register, Value),
    Copy(Register, Register),
    Call(usize, Register),
    Log(Level, &'vm str, Vec<Register>),
    Puts(Vec<Register>),
    CallEq(Register, Register, usize, Register),
    CallNeq(Register, Register, usize, Register),
    // todo if, if_else, unless statements
    IfElse {
        truthy: Register,
        if_scope: usize,
        else_scope: usize,
        output: Register,
    },
    If {
        truthy: Register,
        if_scope: usize,
        output: Register,
    },
    Unless {
        truthy: Register,
        unless_scope: usize,
        output: Register,
    },
    Cast {
        from: Register,
        to: Register,
        rigz_type: RigzType,
    },
    Ret(Register),
    GetVariable(&'vm str, Register),
    LoadLetRegister(&'vm str, Register),
    LoadMutRegister(&'vm str, Register),
    // requires modules, enabled by default
    CallModule {
        module: &'vm str,
        func: &'vm str,
        args: Vec<usize>,
        output: usize,
    },
    CallExtension {
        module: &'vm str,
        this: usize,
        func: &'vm str,
        args: Vec<usize>,
        output: usize,
    },
    /// This instruction will clone your module, ideally modules implement Copy + Clone
    CallVMExtension {
        module: &'vm str,
        func: &'vm str,
        args: Vec<usize>,
        output: usize,
    },
    /// Danger Zone, use these instructions at your own risk (sorted by risk)
    /// in the right situations these will be fantastic, otherwise avoid them

    /// Removes the input value from its current register and replaces the value with None
    UnaryClear(Unary, Clear),
    /// Removes the input value(s) from its/their current register(s) and replaces the value(s) with None
    BinaryClear(Binary, Clear),
    Clear(Clear),
    Goto(usize, usize),
    AddInstruction(usize, Box<Instruction<'vm>>),
    InsertAtInstruction(usize, usize, Box<Instruction<'vm>>),
    UpdateInstruction(usize, usize, Box<Instruction<'vm>>),
    RemoveInstruction(usize, usize),
    /// this assumes no larger registers have been removed, use at your own risk
    Push(Value),
    /// pop the last register and store its value in register
    Pop(Register),
}

impl<'vm> VM<'vm> {
    pub fn handle_clear(&mut self, clear: Clear) -> Result<(), VMError> {
        match clear {
            Clear::One(r) => {
                self.remove_register(r)?;
            }
            Clear::Two(r1, r2) => {
                self.remove_register(r1)?;
                self.remove_register(r2)?;
            }
            Clear::Three(r1, r2, r3) => {
                self.remove_register(r1)?;
                self.remove_register(r2)?;
                self.remove_register(r3)?;
            }
            Clear::Many(reg) => {
                for r in reg {
                    self.remove_register(r)?;
                }
            }
        }
        Ok(())
    }

    #[inline]
    pub fn process_core_instruction(
        &mut self,
        instruction: Instruction<'vm>,
    ) -> Result<VMState, VMError> {
        match instruction {
            Instruction::Halt(r) => return Ok(VMState::Done(self.remove_register_eval_scope(r)?)),
            Instruction::Clear(clear) => self.handle_clear(clear)?,
            Instruction::Unary(u) => self.handle_unary(u)?,
            Instruction::Binary(b) => self.handle_binary(b)?,
            Instruction::UnaryAssign(u) => self.handle_unary_assign(u)?,
            Instruction::BinaryAssign(b) => self.handle_binary_assign(b)?,
            Instruction::UnaryClear(u, clear) => self.handle_unary_clear(u, clear)?,
            Instruction::BinaryClear(b, clear) => self.handle_binary_clear(b, clear)?,
            Instruction::Push(v) => self.stack.push(v),
            Instruction::Pop(r) => match self.stack.pop() {
                None => {
                    return Err(VMError::RuntimeError(format!(
                        "Pop called on empty registers with {}",
                        r
                    )))
                }
                Some(v) => self.insert_register(r, v),
            },
            Instruction::Load(r, v) => self.insert_register(r, v),
            Instruction::LoadLetRegister(name, register) => self.load_let(name, register)?,
            Instruction::LoadMutRegister(name, register) => self.load_mut(name, register)?,
            Instruction::Call(scope_index, register) => self.call_frame(scope_index, register)?,
            Instruction::CallModule {
                module,
                func,
                args,
                output,
            } => {
                let module = self.get_module_clone(module)?;
                let args = self.resolve_registers(args)?;
                let v = module.call(func, args).unwrap_or_else(|e| e.to_value());
                self.insert_register(output, v);
            }
            Instruction::CallExtension {
                module,
                this,
                func,
                args,
                output,
            } => {
                let module = self.get_module_clone(module)?;
                let this = self.resolve_register(this)?;
                let args = self.resolve_registers(args)?;
                let v = module
                    .call_extension(this, func, args)
                    .unwrap_or_else(|e| e.to_value());
                self.insert_register(output, v);
            }
            Instruction::CallVMExtension {
                module,
                func,
                args,
                output,
            } => {
                let module = self.get_module_clone(module)?;
                let args = self.resolve_registers(args)?;
                let value = module
                    .vm_extension(self, func, args)
                    .unwrap_or_else(|e| e.to_value());
                self.insert_register(output, value)
            }
            Instruction::Copy(from, to) => {
                let copy = self.resolve_register(from)?;
                self.insert_register(to, copy);
            }
            Instruction::Cast {
                from,
                rigz_type,
                to,
            } => {
                let value = self.resolve_register(from)?;
                self.insert_register(to, value.cast(rigz_type)?);
            }
            Instruction::CallEq(a, b, scope_index, output) => {
                let a = self.resolve_register(a)?;
                let b = self.resolve_register(b)?;
                if a == b {
                    self.call_frame(scope_index, output)?;
                }
            }
            Instruction::CallNeq(a, b, scope_index, output) => {
                let a = self.resolve_register(a)?;
                let b = self.resolve_register(b)?;
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
                if self.resolve_register(truthy)?.to_bool() {
                    self.call_frame(if_scope, output)?;
                } else {
                    self.call_frame(else_scope, output)?;
                }
            }
            Instruction::If {
                truthy,
                if_scope,
                output,
            } => {
                if self.resolve_register(truthy)?.to_bool() {
                    self.call_frame(if_scope, output)?;
                } else {
                    self.insert_register(output, Value::None)
                }
            }
            Instruction::Unless {
                truthy,
                unless_scope,
                output,
            } => {
                if !self.resolve_register(truthy)?.to_bool() {
                    self.call_frame(unless_scope, output)?;
                } else {
                    self.insert_register(output, Value::None)
                }
            }
            Instruction::GetVariable(name, reg) => match self.current.get_variable(name, self) {
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
            Instruction::Log(level, tmpl, args) => {
                if !self.options.enable_logging {
                    return Ok(VMState::Running);
                }

                let mut res = (*tmpl).to_string();
                for arg in args {
                    let l = self.resolve_register(arg)?.to_string();
                    res = res.replacen("{}", l.as_str(), 1);
                }
                log!(level, "{}", res)
            }
            Instruction::Puts(args) => {
                let mut puts = String::new();
                for r in args {
                    let arg = self.resolve_register(r)?;
                    puts.push_str(", ");
                    puts.push_str(arg.to_string().as_str());
                }
                println!("{}", puts)
            }
            Instruction::Ret(r) => {
                return Err(VMError::UnsupportedOperation(format!(
                    "Ret not handled by parent function: R{}",
                    r
                )))
            }
            Instruction::Goto(scope_id, index) => {
                self.sp = scope_id;
                self.current.pc = index;
            }
            Instruction::AddInstruction(scope, instruction) => match self.scopes.get_mut(scope) {
                None => {
                    return Err(VMError::ScopeDoesNotExist(format!(
                        "Scope does not exist: {}",
                        scope
                    )))
                }
                Some(s) => {
                    s.instructions.push(*instruction);
                }
            },
            Instruction::InsertAtInstruction(scope, index, new_instruction) => {
                match self.scopes.get_mut(scope) {
                    None => {
                        return Err(VMError::ScopeDoesNotExist(format!(
                            "Scope does not exist: {}",
                            scope
                        )))
                    }
                    Some(s) => s.instructions.insert(index, *new_instruction),
                }
            }
            Instruction::UpdateInstruction(scope, index, new_instruction) => {
                match self.scopes.get_mut(scope) {
                    None => {
                        return Err(VMError::ScopeDoesNotExist(format!(
                            "Scope does not exist: {}",
                            scope
                        )))
                    }
                    Some(s) => match s.instructions.get_mut(index) {
                        None => {
                            return Err(VMError::ScopeDoesNotExist(format!(
                                "Scope does not exist: {}",
                                scope
                            )))
                        }
                        Some(i) => {
                            *i = *new_instruction;
                        }
                    },
                }
            }
            Instruction::RemoveInstruction(scope, index) => match self.scopes.get_mut(scope) {
                None => {
                    return Err(VMError::ScopeDoesNotExist(format!(
                        "Scope does not exist: {}",
                        scope
                    )))
                }
                Some(s) => {
                    if index >= s.instructions.len() {
                        return Err(VMError::UnsupportedOperation(format!(
                            "Instruction does not exist: {}#{}",
                            scope, index
                        )));
                    }
                    s.instructions.remove(index);
                }
            },
        };
        Ok(VMState::Running)
    }
}
