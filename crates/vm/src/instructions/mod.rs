mod binary;
mod unary;

use crate::objects::RigzType;
use crate::vm::{RegisterValue, VMState};
use crate::{
    outln, Binary, BinaryAssign, Number, Register, Scope, Unary, UnaryAssign, VMError, Value, VM,
};
use indexmap::IndexMap;
use log::{log, Level};

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
    CallRegister(Register, Register),
    Call {
        scope: usize,
        args: Vec<Register>,
        output: Register,
    },
    CallMemo {
        scope: usize,
        args: Vec<Register>,
        output: Register,
    },
    CallSelf {
        scope: usize,
        this: Register,
        args: Vec<Register>,
        output: Register,
        mutable: bool,
    },
    CallSelfMemo {
        scope: usize,
        this: Register,
        args: Vec<Register>,
        output: Register,
        mutable: bool,
    },
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
    ForList {
        this: Register,
        scope: usize,
        output: Register,
    },
    ForMap {
        this: Register,
        scope: usize,
        key: Register,
        value: Register,
        output: Register,
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
            Instruction::CallRegister(scope_register, output) => {
                let r = match self.get_register(scope_register) {
                    RegisterValue::ScopeId(..) => scope_register,
                    v => {
                        return VMState::error(VMError::UnsupportedOperation(format!(
                            "Call Register called on non-scope value {v:?}"
                        )))
                    }
                };
                let v = self.resolve_register(r);
                self.insert_register(output, v.into());
            }
            Instruction::Call {
                scope,
                output,
                args,
            } => match self.call_frame(scope, args, output) {
                Ok(_) => {}
                Err(e) => return e.into(),
            },
            Instruction::CallSelf {
                scope,
                output,
                this,
                mutable,
                args,
            } => match self.call_frame_self(scope, this, output, args, mutable) {
                Ok(_) => {}
                Err(e) => return e.into(),
            },
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
                    match self.call_frame(scope_index, vec![], output) {
                        Ok(_) => {}
                        Err(e) => return e.into(),
                    };
                }
            }
            Instruction::CallNeq(a, b, scope_index, output) => {
                let a = self.resolve_register(a);
                let b = self.resolve_register(b);
                if a != b {
                    match self.call_frame(scope_index, vec![], output) {
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
                let (scope, r) = if self.resolve_register(truthy).to_bool() {
                    let (if_scope, output) = if_scope;
                    (if_scope, output)
                } else {
                    let (else_scope, r) = else_scope;
                    (else_scope, r)
                };
                let r = self.handle_scope(scope, r, vec![], r);
                self.insert_register(output, r.into());
            }
            Instruction::If {
                truthy,
                if_scope,
                output,
            } => {
                let v = if self.resolve_register(truthy).to_bool() {
                    self.handle_scope(if_scope, output, vec![], output)
                } else {
                    Value::None
                };
                self.insert_register(output, v.into());
            }
            Instruction::Unless {
                truthy,
                unless_scope,
                output,
            } => {
                let v = if self.resolve_register(truthy).to_bool() {
                    self.handle_scope(unless_scope, output, vec![], output)
                } else {
                    Value::None
                };
                self.insert_register(output, v.into());
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
            // why do I have both of these?
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
            Instruction::CallMemo {
                scope,
                output,
                args,
            } => match self.call_frame_memo(scope, args, output) {
                Ok(_) => {}
                Err(e) => return VMState::error(e),
            },
            Instruction::CallSelfMemo {
                scope,
                this,
                output,
                mutable,
                args,
            } => match self.call_frame_self_memo(scope, this, output, args, mutable) {
                Ok(_) => {}
                Err(e) => return VMState::error(e),
            },
            Instruction::ForList {
                this,
                scope,
                output,
            } => {
                let mut result = vec![];
                let this = self.resolve_register(this).to_list();
                for value in this {
                    self.insert_register(output, value.into());
                    // todo ideally this doesn't need a call frame per intermediate, it should be possible to reuse the current scope/fram
                    // the process_ret instruction for the scope is the reason this is needed
                    let value = self.handle_scope(scope, output, vec![output], output);
                    if value != Value::None {
                        result.push(value)
                    }
                }
                self.insert_register(output, result.into());
            }
            Instruction::ForMap {
                this,
                scope,
                key,
                value,
                output,
            } => {
                let mut result = IndexMap::new();
                let this = self.resolve_register(this).to_map();
                for (k, v) in this {
                    self.insert_register(key, k.into());
                    self.insert_register(value, v.into());
                    let value = self.handle_scope(scope, output, vec![key, value], output);
                    match value {
                        Value::None => {}
                        Value::Tuple(mut t) if t.len() == 2 => {
                            let v = t.remove(1);
                            let k = t.remove(0);
                            if k != Value::None && v != Value::None {
                                result.insert(k, v);
                            }
                        }
                        // todo should a single value be both the key & value?
                        _ => {
                            let e = VMError::UnsupportedOperation(format!(
                                "Invalid args in for-map {value}"
                            ))
                            .to_value();
                            result.insert(e.clone(), e);
                        }
                    }
                }
                self.insert_register(output, result.into());
            }
        };
        VMState::Running
    }

    fn instance_get(&mut self, source: Register, attr: Register, output: Register) {
        let attr = self.resolve_register(attr);
        let source = self.resolve_register(source);
        let v = match source.get(attr) {
            Ok(Some(v)) => v,
            Ok(None) => Value::None,
            Err(e) => e.into(),
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
            (Value::Tuple(s), Value::Number(n)) => match n.to_usize() {
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
