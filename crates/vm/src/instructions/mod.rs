mod binary;
mod unary;

use log::{log, Level};
use crate::{outln, Binary, BinaryAssign, Number, Register, Unary, UnaryAssign, VMError, Value, VM};
use crate::objects::RigzType;
use crate::vm::{RegisterValue, VMState};

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
    InstanceGet(Register, Register, Register),
    InstanceSet {
        source: Register,
        index: Register,
        value: Register,
        output: Register,
    },
    InstanceSetMut {
        source: Register,
        index: Register,
        value: Register,
    },
    Copy(Register, Register),
    Move(Register, Register),
    Call(usize, Register),
    CallSelf(usize, Register, Register, bool),
    Log(Level, &'vm str, Vec<Register>),
    Puts(Vec<Register>),
    CallEq(Register, Register, usize, Register),
    CallNeq(Register, Register, usize, Register),
    /// Generally CallSelf should be used instead
    SetSelf(Register, bool),
    GetSelf(Register, bool),
    // todo do I need if, if_else, unless statements, or can I use expressions in the VM?
    IfElse {
        truthy: Register,
        if_scope: (usize, Register),
        else_scope: (usize, Register),
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
    Push(Register),
    Pop(Register),
    Goto(usize, usize),
    AddInstruction(usize, Box<Instruction<'vm>>),
    InsertAtInstruction(usize, usize, Box<Instruction<'vm>>),
    UpdateInstruction(usize, usize, Box<Instruction<'vm>>),
    RemoveInstruction(usize, usize),
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
                    let og = match self.current.borrow().get_mutable_variable("self", self) {
                        Ok(v) => match v {
                            None => {
                                return VMState::Done(
                                    VMError::RuntimeError("Self not set".into()).into(),
                                )
                            }
                            Some(og) => og,
                        },
                        Err(e) => {
                            return VMState::Done(
                                VMError::RuntimeError(format!("Failed to get self: {e:?}")).into(),
                            )
                        }
                    };
                    self.swap_register(og, output);
                } else {
                    let s = match self.current.borrow().get_variable("self", self) {
                        None => {
                            return VMState::Done(
                                VMError::RuntimeError("Self not set".into()).into(),
                            )
                        }
                        Some(s) => s,
                    };
                    let v = self.resolve_register(s);
                    self.insert_register(output, v.into());
                };
            }
            Instruction::Clear(clear) => self.handle_clear(clear),
            Instruction::Unary(u) => self.handle_unary(u),
            Instruction::Binary(b) => self.handle_binary(b),
            Instruction::UnaryAssign(u) => self.handle_unary_assign(u),
            Instruction::BinaryAssign(b) => self.handle_binary_assign(b),
            Instruction::UnaryClear(u, clear) => self.handle_unary_clear(u, clear),
            Instruction::BinaryClear(b, clear) => self.handle_binary_clear(b, clear),
            Instruction::Load(r, v) => {
                self.insert_register(r, v);
            }
            Instruction::LoadLetRegister(name, register) => match self.load_let(name, register) {
                Ok(_) => {}
                Err(e) => return VMState::Done(e.into()),
            },
            Instruction::LoadMutRegister(name, register) => match self.load_mut(name, register) {
                Ok(_) => {}
                Err(e) => return VMState::Done(e.into()),
            },
            Instruction::Call(scope_index, register) => {
                match self.call_frame(scope_index, register) {
                    Ok(_) => {}
                    Err(e) => return e.into(),
                }
            }
            Instruction::CallSelf(scope_index, output, this, mutable) => {
                match self.call_frame_self(scope_index, output, this, mutable) {
                    Ok(_) => {}
                    Err(e) => return e.into(),
                }
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
                        let v = module.call(func, args.into()).unwrap_or_else(|e| e.into());
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
                            .call_extension(this, func, args.into())
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
                            module.call_mutable_extension(v, func, args.clone().into())
                        }) {
                            Ok(Some(v)) => {
                                self.insert_register(output, v.into());
                            }
                            Ok(None) => {}
                            Err(e) => {
                                self.insert_register(output, e.into());
                            }
                        }
                    }
                    Err(e) => {
                        self.insert_register(this, e.into());
                    }
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
                            .vm_extension(self, func, args.into())
                            .unwrap_or_else(|e| e.into());
                        self.insert_register(output, value.into());
                    }
                    Err(e) => {
                        self.insert_register(output, e.into());
                    }
                };
            }
            Instruction::Copy(from, to) => {
                let copy = self.resolve_register(from);
                self.insert_register(to, copy.into());
            }
            Instruction::Move(from, to) => {
                let copy = self.remove_register(from);
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
                    match self.call_frame(scope_index, output) {
                        Ok(_) => {}
                        Err(e) => return e.into(),
                    };
                }
            }
            Instruction::CallNeq(a, b, scope_index, output) => {
                let a = self.resolve_register(a);
                let b = self.resolve_register(b);
                if a != b {
                    match self.call_frame(scope_index, output) {
                        Ok(_) => {}
                        Err(e) => return e.into(),
                    };
                }
            }
            Instruction::IfElse {
                truthy,
                if_scope,
                else_scope,
                output,
            } => {
                let r = if self.resolve_register(truthy).to_bool() {
                    let (if_scope, output) = if_scope;
                    match self.call_frame(if_scope, output) {
                        Ok(_) => {}
                        Err(e) => return e.into(),
                    };
                    output
                } else {
                    let (else_scope, output) = else_scope;
                    match self.call_frame(else_scope, output) {
                        Ok(_) => {}
                        Err(e) => return e.into(),
                    };
                    output
                };
                self.insert_parent_register(output, r.into());
            }
            Instruction::If {
                truthy,
                if_scope,
                output,
            } => {
                if self.resolve_register(truthy).to_bool() {
                    match self.call_frame(if_scope, output) {
                        Ok(_) => {}
                        Err(e) => return e.into(),
                    };
                } else {
                    self.insert_register(output, Value::None.into());
                }
            }
            Instruction::Unless {
                truthy,
                unless_scope,
                output,
            } => {
                if !self.resolve_register(truthy).to_bool() {
                    match self.call_frame(unless_scope, output) {
                        Ok(_) => {}
                        Err(e) => return e.into(),
                    };
                } else {
                    self.insert_register(output, Value::None.into());
                }
            }
            Instruction::GetVariable(name, reg) => {
                let r = self.current.borrow().get_variable(name, self);
                match r {
                    None => {
                        return VMState::Done(
                            VMError::VariableDoesNotExist(format!(
                                "Variable {} does not exist",
                                name
                            ))
                            .into(),
                        );
                    }
                    Some(s) => {
                        let v = self.resolve_register(s);
                        self.insert_register(reg, v.into());
                    }
                }
            }
            Instruction::GetMutableVariable(name, reg) => {
                let og = match self.current.borrow().get_mutable_variable(name, self) {
                    Ok(None) => None,
                    Err(e) => {
                        self.insert_register(reg, e.into());
                        None
                    }
                    Ok(Some(original)) => Some(original),
                };
                match og {
                    None => {
                        return VMState::Done(
                            VMError::VariableDoesNotExist(format!(
                                "Variable {} does not exist",
                                name
                            ))
                            .into(),
                        );
                    }
                    Some(og) => self.swap_register(og, reg),
                }
            }
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
                if args.is_empty() {
                    outln!();
                } else {
                    let mut puts = String::new();
                    let len = args.len() - 1;
                    for (index, r) in args.into_iter().enumerate() {
                        let arg = self.resolve_register(r);
                        puts.push_str(arg.to_string().as_str());
                        if index < len {
                            puts.push_str(", ");
                        }
                    }
                    outln!("{}", puts);
                }
            }
            Instruction::Ret(r) => {
                return VMState::error(VMError::UnsupportedOperation(format!(
                    "Ret not handled by parent function: R{}",
                    r
                )))
            }
            Instruction::Goto(scope_id, index) => {
                self.sp = scope_id;
                self.current.borrow_mut().pc = index;
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
            Instruction::InstanceSet {
                source,
                index,
                value,
                output,
            } => {
                self.instance_set(source, index, value, output);
            }
            Instruction::InstanceSetMut {
                source,
                index,
                value,
            } => {
                self.instance_set(source, index, value, source);
            }
            Instruction::Push(input) => {
                let v = self.resolve_register(input);
                self.stack.push(v);
            }
            Instruction::Pop(output) => {
                let v = match self.stack.pop() {
                    None => VMError::RuntimeError("Pop called on empty stack".into()).into(),
                    Some(v) => v.into(),
                };
                self.insert_register(output, v);
            }
        };
        VMState::Running
    }

    fn instance_get(&mut self, source: Register, attr: Register, output: Register) {
        let attr = self.resolve_register(attr);
        let source = self.resolve_register(source);
        let v = match (source, attr) {
            // todo support ranges as attr
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

    fn instance_set(
        &mut self,
        source: Register,
        index: Register,
        value: Register,
        output: Register,
    ) {
        let attr = self.resolve_register(index);
        let value = self.resolve_register(value);
        let mut source = self.resolve_register(source);
        match (&mut source, attr) {
            // todo support ranges as attr
            (Value::String(s), Value::Number(n)) => match n.to_usize() {
                Ok(index) => {
                    s.insert_str(index, value.to_string().as_str());
                }
                Err(e) => {
                    source = e.into();
                }
            },
            (Value::List(s), Value::Number(n)) => match n.to_usize() {
                Ok(index) => {
                    s.insert(index, value);
                }
                Err(e) => {
                    source = e.into();
                }
            },
            (Value::Map(source), index) => {
                source.insert(index, value);
            }
            (Value::Number(source), Value::Number(n)) => {
                let value = if value.to_bool() { 1 } else { 0 };
                *source = match source {
                    Number::Int(_) => {
                        i64::from_le_bytes((source.to_bits() & (value << n.to_int())).to_le_bytes())
                            .into()
                    }
                    Number::Float(_) => {
                        f64::from_bits(source.to_bits() & (value << n.to_int())).into()
                    }
                }
            }
            (source, attr) => {
                *source =
                    VMError::UnsupportedOperation(format!("Cannot read {} for {}", attr, source))
                        .into();
            }
        };
        self.insert_register(output, source.into());
    }
}
