mod binary;
mod lifecycles;
mod modules;
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
    Load(Register, Value<'vm>),
    Copy(Register, Register),
    Call(usize, Register),
    Log(Level, &'vm str, Vec<Register>),
    Puts(Vec<Register>),
    CallEq(Register, Register, usize, Register),
    CallNeq(Register, Register, usize, Register),
    IfElse {
        truthy: Register,
        if_scope: usize,
        else_scope: usize,
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
        function: &'vm str,
        args: Vec<Register>,
        output: Register,
    },
    CallExtensionModule {
        module: &'vm str,
        function: &'vm str,
        this: Register,
        args: Vec<Register>,
        output: Register,
    },
    // requires Lifecycles, enabled by default
    Publish(Register),
    PublishEvent(&'vm str, Register),
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
    Push(Value<'vm>),
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

    pub fn process_core_instruction(
        &mut self,
        instruction: Instruction<'vm>,
    ) -> Result<VMState<'vm>, VMError> {
        match instruction {
            Instruction::Halt(r) => return Ok(VMState::Done(self.remove_register_eval_scope(r)?)),
            Instruction::Clear(clear) => self.handle_clear(clear)?,
            Instruction::Unary(u) => self.handle_unary(u)?,
            Instruction::Binary(b) => self.handle_binary(b)?,
            Instruction::UnaryAssign(u) => self.handle_unary_assign(u)?,
            Instruction::BinaryAssign(b) => self.handle_binary_assign(b)?,
            Instruction::UnaryClear(u, clear) => self.handle_unary_clear(u, clear)?,
            Instruction::BinaryClear(b, clear) => self.handle_binary_clear(b, clear)?,
            Instruction::Push(v) => match self.registers.len() {
                0 => self.insert_register(2, v),
                k => self.insert_register(k, v),
            },
            Instruction::Pop(r) => match self.registers.pop() {
                None => {
                    return Err(VMError::RuntimeError(format!(
                        "Pop called on empty registers with {}",
                        r
                    )))
                }
                Some((_, v)) => self.insert_register(r, v),
            },
            Instruction::Load(r, v) => self.insert_register(r, v),
            Instruction::LoadLetRegister(name, register) => self.load_let(name, register)?,
            Instruction::LoadMutRegister(name, register) => self.load_mut(name, register)?,
            Instruction::Call(scope_index, register) => self.call_frame(scope_index, register)?,
            Instruction::Publish(value) => return self.publish(value),
            Instruction::PublishEvent(message, value) => return self.publish_event(message, value),
            Instruction::CallModule {
                module,
                function,
                args,
                output,
            } => return self.handle_call_module_instruction(module, function, args, output),
            Instruction::CallExtensionModule {
                module,
                function,
                this,
                args,
                output,
            } => {
                return self
                    .handle_call_extension_module_instruction(module, function, this, args, output)
            }
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
                    let l = self.get_register(arg)?.to_string();
                    res = res.replacen("{}", l.as_str(), 1);
                }
                log!(level, "{}", res)
            }
            Instruction::Puts(args) => {
                let mut puts = String::new();
                for r in args {
                    let arg = self.get_register(r)?;
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
