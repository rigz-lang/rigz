mod binary;
mod unary;

use crate::objects::RigzType;
use crate::vm::{StackValue, VMState};
use crate::{outln, BinaryOperation, Number, UnaryOperation, VMError, Value, Variable, VM};
use indexmap::IndexMap;
use log::{log, Level};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction<'vm> {
    Halt,
    HaltIfError,
    Unary(UnaryOperation),
    Binary(BinaryOperation),
    BinaryAssign(BinaryOperation),
    Load(StackValue),
    InstanceGet(bool),
    InstanceSet,
    InstanceSetMut,
    Call(usize),
    CallMemo(usize),
    Log(Level, &'vm str, usize),
    Puts(usize),
    CallEq(usize),
    CallNeq(usize),
    // todo do I need if, if_else, unless statements, or can I use expressions in the VM?
    IfElse {
        if_scope: usize,
        else_scope: usize,
    },
    If(usize),
    Unless(usize),
    Cast {
        rigz_type: RigzType,
    },
    Ret,
    GetVariable(&'vm str),
    GetMutableVariable(&'vm str),
    GetVariableReference(&'vm str),
    LoadLet(&'vm str),
    LoadMut(&'vm str),
    PersistScope(&'vm str),
    // requires modules, enabled by default
    /// Module instructions will clone your module, ideally modules implement Copy + Clone
    CallModule {
        module: &'vm str,
        func: &'vm str,
        args: usize,
    },
    CallExtension {
        module: &'vm str,
        func: &'vm str,
        args: usize,
    },
    CallMutableExtension {
        module: &'vm str,
        func: &'vm str,
        args: usize,
    },
    CallVMExtension {
        module: &'vm str,
        func: &'vm str,
        args: usize,
    },
    ForList {
        scope: usize,
    },
    ForMap {
        scope: usize,
    },
    /// Danger Zone, use these instructions at your own risk (sorted by risk)
    /// in the right situations these will be fantastic, otherwise avoid them
    Pop(usize),
    Goto(usize, usize),
    AddInstruction(usize, Box<Instruction<'vm>>),
    InsertAtInstruction(usize, usize, Box<Instruction<'vm>>),
    UpdateInstruction(usize, usize, Box<Instruction<'vm>>),
    RemoveInstruction(usize, usize),
}

impl<'vm> VM<'vm> {
    #[inline]
    #[log_derive::logfn_inputs(Debug, fmt = "process_instruction(vm={:#p}, instruction={:?})")]
    pub fn process_core_instruction(&mut self, instruction: &Instruction<'vm>) -> VMState {
        match instruction {
            Instruction::Halt => return VMState::Done(self.next_resolved_value("halt")),
            Instruction::HaltIfError => {
                let value = self.next_resolved_value("halt if error");
                if let Value::Error(e) = value.borrow().deref() {
                    return e.clone().into();
                };
                self.store_value(value.into());
            }
            &Instruction::Unary(u) => self.handle_unary(u),
            &Instruction::Binary(b) => self.handle_binary(b),
            &Instruction::BinaryAssign(b) => self.handle_binary_assign(b),
            Instruction::Load(r) => {
                self.store_value(r.clone());
            }
            Instruction::LoadLet(name) => match self.load_let(name) {
                Ok(_) => {}
                Err(e) => return VMState::Done(e.into()),
            },
            Instruction::LoadMut(name) => match self.load_mut(name) {
                Ok(_) => {}
                Err(e) => return VMState::Done(e.into()),
            },
            Instruction::Call(scope) => match self.call_frame(*scope) {
                Ok(_) => {}
                Err(e) => return e.into(),
            },
            Instruction::CallModule { module, func, args } => {
                match self.get_module_clone(module) {
                    Ok(module) => {
                        let args = self.resolve_args(*args);
                        let v = module.call(func, args.into()).unwrap_or_else(|e| e.into());
                        self.store_value(v.into());
                    }
                    Err(e) => {
                        self.store_value(e.into());
                    }
                };
            }
            Instruction::CallExtension { module, func, args } => {
                match self.get_module_clone(module) {
                    Ok(module) => {
                        let this = self.next_resolved_value("call_extension");
                        let args = self.resolve_args(*args);
                        let v = module
                            .call_extension(this, func, args.into())
                            .unwrap_or_else(|e| e.into());
                        self.store_value(v.into());
                    }
                    Err(e) => {
                        self.store_value(e.into());
                    }
                };
            }
            Instruction::CallMutableExtension { module, func, args } => {
                match self.get_module_clone(module) {
                    Ok(module) => {
                        let this = self.next_resolved_value("call_mut_extension");
                        let args = self.resolve_args(*args);
                        match module.call_mutable_extension(this, func, args.into()) {
                            Ok(Some(v)) => {
                                self.store_value(v.into());
                            }
                            Ok(None) => {}
                            Err(e) => {
                                self.store_value(e.into());
                            }
                        }
                    }
                    Err(e) => {
                        self.store_value(e.into());
                    }
                };
            }
            Instruction::CallVMExtension { module, func, args } => {
                match self.get_module_clone(module) {
                    Ok(module) => {
                        let args = self.resolve_args(*args);
                        let value = module
                            .vm_extension(self, func, args.into())
                            .unwrap_or_else(|e| e.into());
                        self.store_value(value.into());
                    }
                    Err(e) => {
                        self.store_value(e.into());
                    }
                };
            }
            Instruction::PersistScope(var) => {
                let next = self.next_resolved_value("persist_scope");
                self.store_value(next.clone().into());
                let current = self.frames.current.borrow();
                if current.parent.is_none() {
                    return VMError::UnsupportedOperation(format!(
                        "cannot persist scope without parent {current:?}"
                    ))
                    .into();
                };
                let parent = self.frames.current.borrow().find_variable(var, self, None);

                let mut frame = match parent {
                    None => self.frames.current.borrow_mut(),
                    Some(id) => self.frames.frames[id].borrow_mut(),
                };
                let old = match frame.variables.get_mut(var).unwrap() {
                    Variable::Let(v) => v,
                    Variable::Mut(v) => v,
                };
                *old = next.into();
            }
            Instruction::Cast { rigz_type } => {
                let value = self.next_resolved_value("cast");
                self.store_value(value.borrow().cast(rigz_type).into());
            }
            Instruction::CallEq(scope_index) => {
                let b = self.next_resolved_value("call eq - rhs");
                let a = self.next_resolved_value("call eq - lhs");
                if a == b {
                    match self.call_frame(*scope_index) {
                        Ok(_) => {}
                        Err(e) => return e.into(),
                    };
                }
            }
            Instruction::CallNeq(scope_index) => {
                let b = self.next_resolved_value("call neq - rhs");
                let a = self.next_resolved_value("call neq - lhs");
                if a == b {
                    match self.call_frame(*scope_index) {
                        Ok(_) => {}
                        Err(e) => return e.into(),
                    };
                }
            }
            Instruction::IfElse {
                if_scope,
                else_scope,
            } => {
                let truthy = self.next_resolved_value("if else");
                let scope = if truthy.borrow().to_bool() {
                    if_scope
                } else {
                    else_scope
                };
                let v = self.handle_scope(*scope);
                self.store_value(v.into());
            }
            Instruction::If(if_scope) => {
                let truthy = self.next_resolved_value("if");
                let v = if truthy.borrow().to_bool() {
                    self.handle_scope(*if_scope)
                } else {
                    Value::None.into()
                };
                self.store_value(v.into());
            }
            Instruction::Unless(unless_scope) => {
                let truthy = self.next_resolved_value("unless");
                let v = if !truthy.borrow().to_bool() {
                    self.handle_scope(*unless_scope)
                } else {
                    Value::None.into()
                };
                self.store_value(v.into());
            }
            Instruction::GetVariableReference(name) => {
                let r = self.frames.current.borrow().get_variable(name, self);
                match r {
                    None => {
                        return VMError::VariableDoesNotExist(format!(
                            "Variable {} does not exist",
                            name
                        ))
                        .into()
                    }
                    Some(v) => {
                        self.store_value(v);
                    }
                }
            }
            Instruction::GetVariable(name) => {
                let r = self.frames.current.borrow().get_variable(name, self);
                match r {
                    None => {
                        return VMError::VariableDoesNotExist(format!(
                            "Variable {} does not exist",
                            name
                        ))
                        .into()
                    }
                    Some(v) => {
                        let v = v.resolve(self).into();
                        self.store_value(v);
                    }
                }
            }
            &Instruction::GetMutableVariable(name) => {
                let (og, err) = match self
                    .frames
                    .current
                    .borrow()
                    .get_mutable_variable(name, self)
                {
                    Ok(None) => (None, None),
                    Err(e) => (None, Some(e)),
                    Ok(Some(original)) => (Some(original), None),
                };

                if let Some(e) = err {
                    self.store_value(e.into());
                }

                match og {
                    None => {
                        return VMError::VariableDoesNotExist(format!(
                            "Variable {} does not exist",
                            name
                        ))
                        .into();
                    }
                    Some(v) => {
                        let v = v.resolve(self).into();
                        self.store_value(v);
                    }
                }
            }
            Instruction::Log(level, tmpl, args) => {
                if !self.options.enable_logging {
                    return VMState::Running;
                }

                let mut res = (*tmpl).to_string();
                let args = self.resolve_args(*args);
                for arg in args {
                    let l = arg.borrow().to_string();
                    res = res.replacen("{}", l.as_str(), 1);
                }
                log!(*level, "{}", res);
            }
            Instruction::Puts(args) => {
                if args == &0 {
                    outln!();
                } else {
                    let args = self.resolve_args(*args);
                    let mut puts = String::new();
                    let len = args.len() - 1;
                    for (index, arg) in args.into_iter().enumerate() {
                        puts.push_str(arg.borrow().to_string().as_str());
                        if index < len {
                            puts.push_str(", ");
                        }
                    }
                    outln!("{}", puts);
                }
            }
            Instruction::Ret => {
                return VMError::UnsupportedOperation(
                    "Ret not handled by parent function".to_string(),
                )
                .into()
            }
            &Instruction::Goto(scope_id, index) => {
                self.sp = scope_id;
                self.frames.current.borrow_mut().pc = index;
            }
            Instruction::AddInstruction(scope, instruction) => match self.scopes.get_mut(*scope) {
                None => {
                    return VMError::ScopeDoesNotExist(format!("Scope does not exist: {}", scope))
                        .into()
                }
                Some(s) => {
                    s.instructions.push(*instruction.clone());
                }
            },
            Instruction::InsertAtInstruction(scope, index, new_instruction) => {
                match self.scopes.get_mut(*scope) {
                    None => {
                        return VMError::ScopeDoesNotExist(format!(
                            "Scope does not exist: {}",
                            scope
                        ))
                        .into()
                    }
                    Some(s) => s.instructions.insert(*index, *new_instruction.clone()),
                }
            }
            Instruction::UpdateInstruction(scope, index, new_instruction) => {
                match self.scopes.get_mut(*scope) {
                    None => {
                        return VMError::ScopeDoesNotExist(format!(
                            "Scope does not exist: {}",
                            scope
                        ))
                        .into()
                    }
                    Some(s) => match s.instructions.get_mut(*index) {
                        None => {
                            return VMError::ScopeDoesNotExist(format!(
                                "Scope does not exist: {}",
                                scope
                            ))
                            .into()
                        }
                        Some(i) => {
                            *i = *new_instruction.clone();
                        }
                    },
                }
            }
            &Instruction::RemoveInstruction(scope, index) => match self.scopes.get_mut(scope) {
                None => {
                    return VMError::ScopeDoesNotExist(format!("Scope does not exist: {}", scope))
                        .into()
                }
                Some(s) => {
                    if index >= s.instructions.len() {
                        return VMError::UnsupportedOperation(format!(
                            "Instruction does not exist: {}#{}",
                            scope, index
                        ))
                        .into();
                    }
                    s.instructions.remove(index);
                }
            },
            &Instruction::InstanceGet(multiple) => {
                self.instance_get(multiple);
            }
            Instruction::InstanceSet => {
                self.instance_set(false);
            }
            Instruction::InstanceSetMut => {
                self.instance_set(true);
            }
            &Instruction::Pop(output) => {
                for _ in 0..output {
                    let s = self.stack.pop();
                    if s.is_none() {
                        break;
                    }
                }
            }
            &Instruction::CallMemo(scope) => match self.call_frame_memo(scope) {
                Ok(_) => {}
                Err(e) => return e.into(),
            },
            &Instruction::ForList { scope } => {
                let mut result = vec![];
                let this = self.next_resolved_value("for-list").borrow().to_list();
                for value in this {
                    self.stack.push(value.into());
                    // todo ideally this doesn't need a call frame per intermediate, it should be possible to reuse the current scope/fram
                    // the process_ret instruction for the scope is the reason this is needed
                    let value = self.handle_scope(scope);
                    let value = value.borrow().clone();
                    if value != Value::None {
                        result.push(value)
                    }
                }
                self.store_value(result.into());
            }
            &Instruction::ForMap { scope } => {
                let mut result = IndexMap::new();
                let this = self.next_resolved_value("for-map").borrow().to_map();
                for (k, v) in this {
                    self.stack.push(v.into());
                    self.stack.push(k.into());
                    let value = self.handle_scope(scope);
                    let value = value.borrow().clone();
                    match value {
                        Value::None => {}
                        Value::Tuple(mut t) if t.len() >= 2 => {
                            // todo this should be == 2 but same tuple is reused appending to front
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
                self.store_value(result.into());
            }
        };
        VMState::Running
    }

    fn instance_get(&mut self, multiple: bool) {
        let attr = self.next_resolved_value("instance_get - attr");
        let source = self.next_resolved_value("instance_get - source");
        let v = match source.borrow().get(attr.borrow().deref()) {
            Ok(Some(v)) => v,
            Ok(None) => Value::None,
            Err(e) => e.into(),
        };
        if multiple {
            self.store_value(source.into());
        }
        self.store_value(v.into());
    }

    fn instance_set(&mut self, mutable: bool) {
        let value = self.next_resolved_value("instance_set - value");
        let attr = self.next_resolved_value("instance_set - attr");
        let source = self.next_resolved_value("instance_set - source");
        let value = value.borrow();
        let source = if mutable {
            source
        } else {
            source.borrow().clone().into()
        };
        match (source.borrow_mut().deref_mut(), attr.borrow().deref()) {
            // todo support ranges as attr
            (Value::String(s), Value::Number(n)) => match n.to_usize() {
                Ok(index) => {
                    s.insert_str(index, value.to_string().as_str());
                }
                Err(e) => {
                    source.replace(e.into());
                }
            },
            (Value::List(s), Value::Number(n)) | (Value::Tuple(s), Value::Number(n)) => {
                match n.to_usize() {
                    Ok(index) => {
                        s.insert(index, value.clone());
                    }
                    Err(e) => {
                        source.replace(e.into());
                    }
                }
            }
            (Value::Map(source), index) => {
                source.insert(index.clone(), value.clone());
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
        if !mutable {
            self.store_value(source.into());
        }
    }
}
