mod binary;
mod unary;

pub use binary::{Binary, BinaryAssign, BinaryOperation};
use log::{log, Level};
pub use unary::{Unary, UnaryAssign, UnaryOperation};

use crate::vm::{RegisterValue, VMState};
use crate::{Register, RigzType, VMError, Value, VM};

// todo simplify clear usage
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
    HaltIfError(Register),
    Unary(Unary),
    Binary(Binary),
    UnaryAssign(UnaryAssign),
    BinaryAssign(BinaryAssign),
    Load(Register, RegisterValue),
    InstanceGet(Register, Value, Register),
    InstanceGetRegister(Register, Register, Register),
    Copy(Register, Register),
    Call(usize, Register),
    CallSelf(usize, Register, Register, bool),
    Log(Level, &'vm str, Vec<Register>),
    Puts(Vec<Register>),
    CallEq(Register, Register, usize, Register),
    CallNeq(Register, Register, usize, Register),
    SetSelf(Register, bool),
    GetSelf(Register, bool),
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
    GetMutableVariable(&'vm str, Register),
    LoadLetRegister(&'vm str, Register),
    LoadMutRegister(&'vm str, Register),
    // requires modules, enabled by default
    /// Module instructions will clone your module, ideally modules implement Copy + Clone
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
    CallMutableExtension {
        module: &'vm str,
        this: usize,
        func: &'vm str,
        args: Vec<usize>,
        output: usize,
    },
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
    Push(RegisterValue),
    /// pop the last register and store its value in register
    Pop(Register),
}

impl<'vm> VM<'vm> {
    pub fn handle_clear(&mut self, clear: Clear) {
        match clear {
            Clear::One(r) => {
                self.remove_register(r);
            }
            Clear::Two(r1, r2) => {
                self.remove_register(r1);
                self.remove_register(r2);
            }
            Clear::Three(r1, r2, r3) => {
                self.remove_register(r1);
                self.remove_register(r2);
                self.remove_register(r3);
            }
            Clear::Many(reg) => {
                for r in reg {
                    self.remove_register(r);
                }
            }
        }
    }

    #[inline]
    pub fn process_core_instruction(&mut self, instruction: Instruction<'vm>) -> VMState {
        match instruction {
            Instruction::Halt(r) => return VMState::Done(self.resolve_register(r)),
            Instruction::HaltIfError(r) => {
                let value = self.resolve_register(r);
                if let Value::Error(e) = value {
                    return VMState::Done(e.into());
                }
            }
            Instruction::SetSelf(register, mutable) => {
                let success = if mutable {
                    self.load_mut("self", register)
                } else {
                    self.load_let("self", register)
                };
                match success {
                    Ok(_) => {}
                    Err(e) => {
                        return VMState::Done(
                            VMError::RuntimeError(format!("Failed to set self: {e:?}")).into(),
                        )
                    }
                }
            }
            Instruction::GetSelf(output, mutable) => {
                if mutable {
                    match self.current.get_mutable_variable("self", self) {
                        Ok(v) => match v {
                            None => {
                                return VMState::Done(
                                    VMError::RuntimeError("Self not set".into()).into(),
                                )
                            }
                            Some(og) => {
                                self.swap_register(og, output);
                            }
                        },
                        Err(e) => {
                            return VMState::Done(
                                VMError::RuntimeError(format!("Failed to get self: {e:?}")).into(),
                            )
                        }
                    }
                } else {
                    match self.current.get_variable("self", self) {
                        None => {
                            return VMState::Done(
                                VMError::RuntimeError("Self not set".into()).into(),
                            )
                        }
                        Some(s) => self.insert_register(output, RegisterValue::Register(s)),
                    }
                };
            }
            Instruction::Clear(clear) => self.handle_clear(clear),
            Instruction::Unary(u) => self.handle_unary(u),
            Instruction::Binary(b) => self.handle_binary(b),
            Instruction::UnaryAssign(u) => self.handle_unary_assign(u),
            Instruction::BinaryAssign(b) => self.handle_binary_assign(b),
            Instruction::UnaryClear(u, clear) => self.handle_unary_clear(u, clear),
            Instruction::BinaryClear(b, clear) => self.handle_binary_clear(b, clear),
            Instruction::Push(v) => self.stack.push(v),
            Instruction::Pop(r) => match self.stack.pop() {
                None => self.insert_register(
                    r,
                    VMError::RuntimeError(format!("Pop called on empty registers with {}", r))
                        .into(),
                ),
                Some(v) => self.insert_register(r, v),
            },
            Instruction::Load(r, v) => self.insert_register(r, v),
            Instruction::LoadLetRegister(name, register) => match self.load_let(name, register) {
                Ok(_) => {}
                Err(e) => return VMState::Done(e.into()),
            },
            Instruction::LoadMutRegister(name, register) => match self.load_mut(name, register) {
                Ok(_) => {}
                Err(e) => return VMState::Done(e.into()),
            },
            Instruction::Call(scope_index, register) => self.call_frame(scope_index, register),
            Instruction::CallSelf(scope_index, output, this, mutable) => {
                self.call_frame_self(scope_index, output, this, mutable)
            }
            Instruction::CallModule {
                module,
                func,
                args,
                output,
            } => {
                match self.get_module_clone(module) {
                    Ok(module) => {
                        let args = self.resolve_registers(args);
                        let v = module.call(func, args).unwrap_or_else(|e| e.into());
                        self.insert_register(output, v.into());
                    }
                    Err(e) => {
                        self.insert_register(output, e.into());
                    }
                };
            }
            Instruction::CallExtension {
                module,
                this,
                func,
                args,
                output,
            } => {
                match self.get_module_clone(module) {
                    Ok(module) => {
                        let this = self.resolve_register(this);
                        let args = self.resolve_registers(args);
                        let v = module
                            .call_extension(this, func, args)
                            .unwrap_or_else(|e| e.into());
                        self.insert_register(output, v.into());
                    }
                    Err(e) => {
                        self.insert_register(output, e.into());
                    }
                };
            }
            Instruction::CallMutableExtension {
                module,
                this,
                func,
                args,
                output,
            } => {
                match self.get_module_clone(module) {
                    Ok(module) => {
                        let args = self.resolve_registers(args);
                        match self.update_register(this, |v| {
                            // todo remove args.clone
                            module.call_mutable_extension(v, func, args.clone())
                        }) {
                            Ok(Some(v)) => self.insert_register(output, v.into()),
                            Ok(None) => {}
                            Err(e) => self.insert_register(output, e.into()),
                        }
                    }
                    Err(e) => self.insert_register(this, e.into()),
                };
            }
            Instruction::CallVMExtension {
                module,
                func,
                args,
                output,
            } => {
                match self.get_module_clone(module) {
                    Ok(module) => {
                        let args = self.resolve_registers(args);
                        let value = module
                            .vm_extension(self, func, args)
                            .unwrap_or_else(|e| e.into());
                        self.insert_register(output, value.into())
                    }
                    Err(e) => {
                        self.insert_register(output, e.into());
                    }
                };
            }
            Instruction::Copy(from, to) => {
                let copy = self.get_register(from);
                self.insert_register(to, copy);
            }
            Instruction::Cast {
                from,
                rigz_type,
                to,
            } => {
                let value = self.resolve_register(from);
                self.insert_register(to, value.cast(rigz_type).into());
            }
            Instruction::CallEq(a, b, scope_index, output) => {
                let a = self.resolve_register(a);
                let b = self.resolve_register(b);
                if a == b {
                    self.call_frame(scope_index, output);
                }
            }
            Instruction::CallNeq(a, b, scope_index, output) => {
                let a = self.resolve_register(a);
                let b = self.resolve_register(b);
                if a != b {
                    self.call_frame(scope_index, output);
                }
            }
            Instruction::IfElse {
                truthy,
                if_scope,
                else_scope,
                output,
            } => {
                if self.resolve_register(truthy).to_bool() {
                    self.call_frame(if_scope, output)
                } else {
                    self.call_frame(else_scope, output)
                }
            }
            Instruction::If {
                truthy,
                if_scope,
                output,
            } => {
                if self.resolve_register(truthy).to_bool() {
                    self.call_frame(if_scope, output);
                } else {
                    self.insert_register(output, Value::None.into())
                }
            }
            Instruction::Unless {
                truthy,
                unless_scope,
                output,
            } => {
                if !self.resolve_register(truthy).to_bool() {
                    self.call_frame(unless_scope, output);
                } else {
                    self.insert_register(output, Value::None.into())
                }
            }
            Instruction::GetVariable(name, reg) => match self.current.get_variable(name, self) {
                None => {
                    self.insert_register(
                        reg,
                        VMError::VariableDoesNotExist(format!("Variable {} does not exist", name))
                            .into(),
                    );
                }
                Some(s) => {
                    let v = self.get_register(s);
                    self.insert_register(reg, v.clone());
                }
            },
            Instruction::GetMutableVariable(name, reg) => match self
                .current
                .get_mutable_variable(name, self)
            {
                Ok(None) => {
                    self.insert_register(
                        reg,
                        VMError::VariableDoesNotExist(format!("Variable {} does not exist", name))
                            .into(),
                    );
                }
                Err(e) => {
                    self.insert_register(reg, e.into());
                }
                Ok(Some(original)) => {
                    self.swap_register(original, reg);
                }
            },
            Instruction::Log(level, tmpl, args) => {
                if !self.options.enable_logging {
                    return VMState::Running;
                }

                let mut res = (*tmpl).to_string();
                for arg in args {
                    let l = self.resolve_register(arg).to_string();
                    res = res.replacen("{}", l.as_str(), 1);
                }
                log!(level, "{}", res)
            }
            Instruction::Puts(args) => {
                let mut puts = String::new();
                for r in args {
                    let arg = self.resolve_register(r);
                    puts.push_str(", ");
                    puts.push_str(arg.to_string().as_str());
                }
                println!("{}", puts)
            }
            Instruction::Ret(r) => {
                return VMState::error(VMError::UnsupportedOperation(format!(
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
                    return VMState::error(VMError::ScopeDoesNotExist(format!(
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
                        return VMState::error(VMError::ScopeDoesNotExist(format!(
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
                        return VMState::error(VMError::ScopeDoesNotExist(format!(
                            "Scope does not exist: {}",
                            scope
                        )))
                    }
                    Some(s) => match s.instructions.get_mut(index) {
                        None => {
                            return VMState::error(VMError::ScopeDoesNotExist(format!(
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
                    return VMState::error(VMError::ScopeDoesNotExist(format!(
                        "Scope does not exist: {}",
                        scope
                    )))
                }
                Some(s) => {
                    if index >= s.instructions.len() {
                        return VMState::error(VMError::UnsupportedOperation(format!(
                            "Instruction does not exist: {}#{}",
                            scope, index
                        )));
                    }
                    s.instructions.remove(index);
                }
            },
            Instruction::InstanceGet(source, attr, output) => {
                self.instance_get(source, attr, output);
            }
            Instruction::InstanceGetRegister(source, attr, output) => {
                let attr = self.resolve_register(attr);
                self.instance_get(source, attr, output);
            }
        };
        VMState::Running
    }

    fn instance_get(&mut self, source: Register, attr: Value, output: Register) {
        let source = self.resolve_register(source);
        let v = match (source, attr) {
            (Value::String(source), Value::Number(n)) => match n.to_usize() {
                Ok(index) => match source.chars().nth(index) {
                    None => VMError::UnsupportedOperation(format!(
                        "Cannot read {}th index of {}",
                        index, source
                    ))
                    .into(),
                    Some(c) => Value::String(c.to_string()),
                },
                Err(e) => e.into(),
            },
            (Value::List(source), Value::Number(n)) => match n.to_usize() {
                Ok(index) => match source.get(index) {
                    None => VMError::UnsupportedOperation(format!(
                        "Cannot read {}th index of {:?}",
                        index, source
                    ))
                    .into(),
                    Some(c) => c.clone(),
                },
                Err(e) => e.into(),
            },
            (Value::Map(source), index) => match source.get(&index) {
                None => VMError::UnsupportedOperation(format!(
                    "Cannot read {} index of {:?}",
                    index, source
                ))
                .into(),
                Some(c) => c.clone(),
            },
            (Value::Number(source), Value::Number(n)) => {
                Value::Bool(source.to_bits() & (1 << n.to_int()) != 0)
            }
            (source, attr) => {
                VMError::UnsupportedOperation(format!("Cannot read {} for {}", attr, source)).into()
            }
        };
        self.insert_register(output, v.into());
    }
}
