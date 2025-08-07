use crate::{err, errln, out, outln, CallFrame, Instruction, MatchArm, Modules, Scope, VMOptions, VMState};
use log::log;
use rigz_core::{
    AsPrimitive, BinaryOperation, EnumDeclaration, IndexMap, Logical, Module, ObjectValue,
    PrimitiveValue, Reference, ResolveValue, ResolvedValue, Reverse, RigzArgs, RigzObject,
    StackValue, UnaryOperation, VMError,
};
use std::cell::RefCell;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::string::ToString;
use std::sync::Arc;

#[macro_export]
macro_rules! runner_common {
    () => {
        #[inline]
        fn next_value<T: Display, F>(&mut self, location: F) -> StackValue
        where
            F: FnOnce() -> T,
        {
            self.stack.next_value(location)
        }

        #[inline]
        fn store_value(&mut self, value: StackValue) {
            self.stack.store_value(value)
        }

        #[inline]
        fn pop(&mut self) -> Option<StackValue> {
            self.stack.pop()
        }

        #[inline]
        fn options(&self) -> &VMOptions {
            &self.options
        }

        #[inline]
        fn modules(&self) -> Modules {
            self.modules.clone()
        }

        #[inline]
        fn get_module(&mut self, module: usize) -> Option<ResolvedModule> {
            let e = match self.modules.get(module) {
                None => VMError::InvalidModule(module.to_string()),
                Some(m) => {
                    return Some(m.clone());
                }
            };
            self.store_value(e.into());
            None
        }

        #[logfn(Trace)]
        #[logfn_inputs(
            Trace,
            fmt = "find_variable(frame={:#p} name={}, vm={:#p}, parent={:?})"
        )]
        fn find_variable(
            &self,
            name: usize,
            frame: &CallFrame,
            parent: Option<usize>,
        ) -> Option<Option<usize>> {
            match frame.variables.contains_key(&name) {
                false => match frame.parent {
                    None => None,
                    Some(parent) => {
                        let frame = self.frames[parent].borrow();
                        self.find_variable(name, &frame, Some(parent))
                    }
                },
                true => Some(parent),
            }
        }

        #[inline]
        fn persist_scope(&mut self, var: usize) -> Option<VMError> {
            let next = self.next_resolved_value(|| "persist_scope");
            self.store_value(next.clone().into());
            let current = self.frames.current.borrow();
            let owner = self.find_variable(var, current.deref(), None);

            let mut frame = match owner {
                None => {
                    return Some(VMError::runtime(format!("{var} not found in callstack")).into())
                }
                Some(None) => self.frames.current.borrow_mut(),
                Some(Some(id)) => self.frames.frames[id].borrow_mut(),
            };
            let old = match frame.variables.get_mut(&var).unwrap() {
                Variable::Let(v) => v,
                Variable::Mut(v) => v,
            };
            *old = next.into();
            None
        }

        #[inline]
        fn load_mut(&mut self, name: usize, shadow: bool) -> Result<(), VMError> {
            let v = self.next_value(|| format!("load_mut - {name}"));
            if shadow {
                self.frames.load_mut(name, v, shadow)
            } else {
                if let Some(og) = self.frames.get_variable(name) {
                    let ogv = og.resolve(self);
                    let sv = v.resolve(self);
                    if Rc::ptr_eq(&ogv, &sv) {
                        return self.frames.load_mut(name, v, shadow);
                    }

                    ogv.swap(&v.resolve(self));
                    self.frames.load_mut(name, og, shadow)
                } else {
                    self.frames.load_mut(name, v, shadow)
                }
            }
        }

        #[inline]
        fn load_let(&mut self, name: usize, shadow: bool) -> Result<(), VMError> {
            let v = self.next_value(|| format!("load_let - {name}"));
            self.frames.load_let(name, v, shadow)
        }

        #[inline]
        fn get_variable(&mut self, name: usize) {
            let r = self.frames.get_variable(name);
            let v = match r {
                None => VMError::VariableDoesNotExist(format!("Variable {} does not exist", name))
                    .into(),
                Some(v) => v.resolve(self).into(),
            };
            self.store_value(v);
        }

        #[inline]
        fn get_mutable_variable(&mut self, name: usize) {
            let og = match self.frames.get_mutable_variable(name) {
                Ok(None) => None,
                Err(e) => Some(e.into()),
                Ok(Some(original)) => Some(original),
            };

            let v = match og {
                None => VMError::VariableDoesNotExist(format!(
                    "Mutable variable {} does not exist",
                    name
                ))
                .into(),
                Some(v) => v.resolve(self).into(),
            };
            self.store_value(v);
        }

        #[inline]
        fn get_variable_reference(&mut self, name: usize) {
            let r = self.frames.get_variable(name);
            let v = match r {
                None => VMError::VariableDoesNotExist(format!(
                    "Variable Reference {} does not exist",
                    name
                ))
                .into(),
                Some(v) => v,
            };
            self.store_value(v);
        }

        #[inline]
        fn call_extension(
            &mut self,
            module: ResolvedModule,
            func: &str,
            args: usize,
        ) -> Result<ObjectValue, VMError> {
            let args = self.resolve_args(args).into();
            let this = self.next_resolved_value(|| "call_extension");
            module.call_extension(this, func, args)
        }

        #[inline]
        fn call_mutable_extension(
            &mut self,
            module: ResolvedModule,
            func: &str,
            args: usize,
        ) -> Result<Option<ObjectValue>, VMError> {
            let args = self.resolve_args(args).into();
            let this = self.next_resolved_value(|| "call_extension");
            module.call_mutable_extension(this, func, args)
        }
    };
}

use std::time::Duration;

#[inline]
pub fn eval_unary(unary_operation: UnaryOperation, val: &ObjectValue) -> ObjectValue {
    match unary_operation {
        UnaryOperation::Neg => -val,
        UnaryOperation::Not => !val,
        UnaryOperation::PrintLn => {
            outln!("{}", val);
            ObjectValue::default()
        }
        UnaryOperation::EPrintLn => {
            errln!("{}", val);
            ObjectValue::default()
        }
        UnaryOperation::Print => {
            out!("{}", val);
            ObjectValue::default()
        }
        UnaryOperation::EPrint => {
            err!("{}", val);
            ObjectValue::default()
        }
        UnaryOperation::Reverse => Reverse::reverse(val),
    }
}

#[inline]
pub fn eval_binary_operation(
    binary_operation: BinaryOperation,
    lhs: &ObjectValue,
    rhs: &ObjectValue,
) -> ObjectValue {
    match binary_operation {
        BinaryOperation::Add => lhs + rhs,
        BinaryOperation::Sub => lhs - rhs,
        BinaryOperation::Shr => lhs >> rhs,
        BinaryOperation::Shl => lhs << rhs,
        BinaryOperation::Eq => (lhs == rhs).into(),
        BinaryOperation::Neq => (lhs != rhs).into(),
        BinaryOperation::Mul => lhs * rhs,
        BinaryOperation::Div => lhs / rhs,
        BinaryOperation::Rem => lhs % rhs,
        BinaryOperation::BitOr => lhs | rhs,
        BinaryOperation::BitAnd => lhs & rhs,
        BinaryOperation::BitXor => lhs ^ rhs,
        BinaryOperation::And => lhs.and(rhs),
        BinaryOperation::Or => lhs.or(rhs),
        BinaryOperation::Xor => lhs.xor(rhs),
        BinaryOperation::Gt => (lhs > rhs).into(),
        BinaryOperation::Gte => (lhs >= rhs).into(),
        BinaryOperation::Lt => (lhs < rhs).into(),
        BinaryOperation::Lte => (lhs <= rhs).into(),
        BinaryOperation::Elvis => if lhs.is_value() {
            lhs.clone()
        } else {
            rhs.clone()
        },
    }
}

#[cfg(feature = "threaded")]
pub type ResolvedModule = Reference<dyn Module + Send + Sync>;

pub enum CallType<'c> {
    Create,
    Call(&'c str),
}

#[cfg(not(feature = "threaded"))]
pub type ResolvedModule = Reference<dyn Module>;

pub trait Runner: ResolveValue {

    fn store_value(&mut self, value: StackValue);

    fn pop(&mut self) -> Option<StackValue>;

    fn next_value<T: Display, F>(&mut self, location: F) -> StackValue
    where
        F: FnOnce() -> T;

    fn options(&self) -> &VMOptions;

    fn update_scope<F>(&mut self, index: usize, update: F) -> Result<(), VMError>
    where
        F: FnMut(&mut Scope) -> Result<(), VMError>;

    fn modules(&self) -> Modules;

    fn get_module(&mut self, module: usize) -> Option<ResolvedModule>;

    fn load_mut(&mut self, name: usize, shadow: bool) -> Result<(), VMError>;
    fn load_let(&mut self, name: usize, shadow: bool) -> Result<(), VMError>;

    fn find_variable(
        &self,
        name: usize,
        frame: &CallFrame,
        parent: Option<usize>,
    ) -> Option<Option<usize>>;

    // using this to distinguish VM runtime self vs rust self
    #[inline]
    fn set_this(&mut self, mutable: bool) -> Result<(), VMError> {
        if mutable {
            self.load_mut(0, true)
        } else {
            self.load_let(0, false)
        }
    }

    fn call_frame(&mut self, scope_index: usize) -> Result<(), VMError>;

    fn call_frame_memo(&mut self, scope_index: usize) -> Result<(), VMError>;

    fn call_dependency(
        &mut self,
        arg: RigzArgs,
        dep: usize,
        call_type: CallType,
    ) -> Result<ObjectValue, VMError>;

    #[inline]
    fn apply_unary(&mut self, unary_operation: UnaryOperation, val: Rc<RefCell<ObjectValue>>) {
        let val = eval_unary(unary_operation, val.borrow().deref());
        self.store_value(val.into());
    }

    #[inline]
    fn handle_unary(&mut self, op: UnaryOperation) {
        let val = self.next_resolved_value(|| "handle_unary");
        self.apply_unary(op, val);
    }

    #[inline]
    fn apply_binary(
        &mut self,
        binary_operation: BinaryOperation,
        lhs: Rc<RefCell<ObjectValue>>,
        rhs: Rc<RefCell<ObjectValue>>,
    ) {
        let v = eval_binary_operation(binary_operation, lhs.borrow().deref(), rhs.borrow().deref());
        self.store_value(v.into())
    }

    #[inline]
    fn handle_binary(&mut self, op: BinaryOperation) {
        let rhs = self.next_resolved_value(|| "handle_binary - rhs");
        let lhs = self.next_resolved_value(|| "handle_binary - lhs");
        self.apply_binary(op, lhs, rhs);
    }

    #[inline]
    fn handle_binary_assign(&mut self, op: BinaryOperation) {
        let rhs = self.next_resolved_value(|| "handle_binary_assign - rhs");
        let lhs = self.next_resolved_value(|| "handle_binary_assign - lhs");
        let v = eval_binary_operation(op, lhs.borrow().deref(), rhs.borrow().deref());
        *lhs.borrow_mut().deref_mut() = v;
    }

    #[inline]
    fn next_resolved_value<T: Display, F>(&mut self, location: F) -> Rc<RefCell<ObjectValue>>
    where
        F: FnOnce() -> T,
    {
        self.next_value(location).resolve(self)
    }

    #[inline]
    fn resolve_args(&mut self, count: usize) -> Vec<Rc<RefCell<ObjectValue>>> {
        (0..count)
            .map(|_| self.next_resolved_value(|| "resolve_args"))
            .collect()
    }

    fn persist_scope(&mut self, var: usize) -> Option<VMError>;

    fn goto(&mut self, scope_id: usize, pc: usize) -> Result<(), VMError>;

    fn send(&mut self, args: usize) -> Result<(), VMError>;

    fn receive(&mut self, args: usize) -> Result<(), VMError>;

    fn spawn(&mut self, scope_id: usize, timeout: Option<usize>) -> Result<(), VMError>;

    fn get_variable(&mut self, name: usize);

    fn get_mutable_variable(&mut self, name: usize);

    fn get_variable_reference(&mut self, name: usize);

    fn find_enum(&mut self, enum_type: usize) -> Result<Arc<EnumDeclaration>, VMError>;

    fn call(
        &mut self,
        module: ResolvedModule,
        func: &str,
        args: usize,
    ) -> Result<ObjectValue, VMError>;

    fn call_extension(
        &mut self,
        module: ResolvedModule,
        func: &str,
        args: usize,
    ) -> Result<ObjectValue, VMError>;

    fn call_mutable_extension(
        &mut self,
        module: ResolvedModule,
        func: &str,
        args: usize,
    ) -> Result<Option<ObjectValue>, VMError>;

    fn call_loop(&mut self, scope_id: usize) -> Option<VMState>;

    fn call_for(&mut self, scope_id: usize) -> Option<VMState>;

    fn call_for_comprehension<T, I, F>(&mut self, scope_id: usize, init: I, save: F) -> Result<T, VMState> where F: FnMut(&mut T, ObjectValue) -> Option<VMError>, I: FnOnce(usize) -> T;
    // fn vm_extension(
    //     &mut self,
    //     module: ResolvedModule,
    //     func: String,
    //     args: usize,
    // ) -> Result<ObjectValue, VMError>;

    // todo this should always be interruptible (if called by run_within, amount of time slept should be removed)
    fn sleep(&self, duration: Duration);

    fn exit<V>(&mut self, value: V)
    where
        V: Into<StackValue>;

    #[allow(unused_variables)]
    #[inline]
    #[log_derive::logfn_inputs(Debug, fmt = "process_instruction(vm={:#p}, instruction={:?})")]
    fn process_core_instruction(&mut self, instruction: &Instruction) -> VMState {
        match instruction {
            Instruction::Halt => {
                let v = self
                    .pop()
                    .map(|e| e.resolve(self))
                    .unwrap_or_else(|| ObjectValue::default().into());
                return VMState::Done(v);
            }
            Instruction::Exit => {
                let v = self
                    .pop()
                    .map(|e| e.resolve(self))
                    .unwrap_or_else(|| ObjectValue::default().into());
                self.exit(v.clone());
                return VMState::Done(v);
            }
            Instruction::HaltIfError => {
                let value = self
                    .pop()
                    .map(|e| e.resolve(self))
                    .unwrap_or_else(|| ObjectValue::default().into());
                if let ObjectValue::Primitive(PrimitiveValue::Error(e)) = value.borrow().deref() {
                    return e.clone().into();
                };
                let s: StackValue = value.into();
                self.store_value(s);
            }
            &Instruction::Unary(u) => self.handle_unary(u),
            &Instruction::Binary(b) => self.handle_binary(b),
            &Instruction::BinaryAssign(b) => self.handle_binary_assign(b),
            Instruction::Load(r) => {
                self.store_value(r.clone().into());
            }
            &Instruction::LoadLet(name, shadow) => {
                if let Err(e) = self.load_let(name, shadow) {
                    return e.into();
                }
            }
            &Instruction::LoadMut(name, shadow) => {
                if let Err(e) = self.load_mut(name, shadow) {
                    return e.into();
                }
            }
            &Instruction::Call(scope) => {
                if let Err(e) = self.call_frame(scope) {
                    return e.into();
                }
            }
            Instruction::CallModule { module, func, args } => {
                if let Some(module) = self.get_module(*module) {
                    let v = self.call(module, func, *args).unwrap_or_else(|e| e.into());
                    self.store_value(v.into());
                };
            }
            Instruction::CallExtension { module, func, args } => {
                if let Some(module) = self.get_module(*module) {
                    let v = self
                        .call_extension(module, func, *args)
                        .unwrap_or_else(|e| e.into());
                    self.store_value(v.into());
                };
            }
            Instruction::CallMutableExtension { module, func, args } => {
                if let Some(module) = self.get_module(*module) {
                    match self.call_mutable_extension(module, func, *args) {
                        Ok(Some(v)) => {
                            self.store_value(v.into());
                        }
                        Ok(None) => {}
                        Err(e) => {
                            self.store_value(e.into());
                        }
                    }
                }
            }
            // Instruction::CallVMExtension { module, func, args } => {
            //     if let Some(module) = self.get_module(module) {
            //         let value = self
            //             .vm_extension(module, func, args)
            //             .unwrap_or_else(|e| e.into());
            //         self.store_value(value.into());
            //     };
            // }
            &Instruction::PersistScope(var) => {
                if let Some(s) = self.persist_scope(var) {
                    return s.into();
                }
            }
            Instruction::Cast { rigz_type } => {
                let value = self.next_resolved_value(|| "cast");
                self.store_value(value.borrow().cast(rigz_type).into());
            }
            &Instruction::CallEq(scope_index) => {
                let b = self.next_resolved_value(|| "call eq - rhs");
                let a = self.next_resolved_value(|| "call eq - lhs");
                if a == b {
                    if let Err(e) = self.call_frame(scope_index) {
                        return e.into();
                    };
                }
            }
            &Instruction::CallNeq(scope_index) => {
                let b = self.next_resolved_value(|| "call neq - rhs");
                let a = self.next_resolved_value(|| "call neq - lhs");
                if a == b {
                    if let Err(e) = self.call_frame(scope_index) {
                        return e.into();
                    };
                }
            }
            &Instruction::IfElse {
                if_scope,
                else_scope,
            } => {
                let truthy = self.next_resolved_value(|| "if else");
                let scope = if truthy.borrow().to_bool() {
                    if_scope
                } else {
                    else_scope
                };
                match self.handle_scope(scope) {
                    ResolvedValue::Break => return VMState::Break,
                    ResolvedValue::Next => return VMState::Next,
                    ResolvedValue::Value(v) => self.store_value(v.into()),
                    ResolvedValue::Done(v) => return VMState::Done(v),
                };
            }
            &Instruction::If(if_scope) => {
                let truthy = self.next_resolved_value(|| "if");
                let v = if truthy.borrow().to_bool() {
                    match self.handle_scope(if_scope) {
                        ResolvedValue::Break => return VMState::Break,
                        ResolvedValue::Next => return VMState::Next,
                        ResolvedValue::Value(v) => v,
                        ResolvedValue::Done(v) => return VMState::Done(v),
                    }
                } else {
                    ObjectValue::default().into()
                };
                self.store_value(v.into());
            }
            &Instruction::Unless(unless_scope) => {
                let truthy = self.next_resolved_value(|| "unless");
                let v = if !truthy.borrow().to_bool() {
                    match self.handle_scope(unless_scope) {
                        ResolvedValue::Break => return VMState::Break,
                        ResolvedValue::Next => return VMState::Next,
                        ResolvedValue::Value(v) => v,
                        ResolvedValue::Done(v) => return VMState::Done(v),
                    }
                } else {
                    ObjectValue::default().into()
                };
                self.store_value(v.into());
            }
            &Instruction::GetVariableReference(name) => self.get_variable_reference(name),
            &Instruction::GetVariable(name) => self.get_variable(name),
            &Instruction::GetMutableVariable(name) => self.get_mutable_variable(name),
            Instruction::Log(level, tmpl, args) => {
                if !self.options().enable_logging {
                    return VMState::Running;
                }

                let mut res = tmpl.to_string();
                let args = self.resolve_args(*args);
                for arg in args {
                    let l = arg.borrow().to_string();
                    res = res.replacen("{}", l.as_str(), 1);
                }
                log!(*level, "{}", res);
                self.store_value(ObjectValue::default().into());
            }
            &Instruction::Puts(args) => {
                if args == 0 {
                    outln!();
                } else {
                    let args = self.resolve_args(args);
                    for arg in args {
                        let s = arg.borrow().to_string();
                        outln!("{}", s);
                    }
                }
                self.store_value(ObjectValue::default().into());
            }
            Instruction::Ret => {
                return VMError::UnsupportedOperation(format!(
                    "Ret not handled by parent function - {}",
                    self.location()
                ))
                .into()
            }
            &Instruction::Goto(scope_id, index) => match self.goto(scope_id, index) {
                Ok(_) => {}
                Err(e) => return e.into(),
            },
            Instruction::AddInstruction(scope, instruction) => {
                let updated = self.update_scope(*scope, |s| {
                    s.instructions.push(*instruction.clone());
                    Ok(())
                });
                if let Err(e) = updated {
                    return e.into();
                }
            }
            Instruction::InsertAtInstruction(scope, index, new_instruction) => {
                let updated = self.update_scope(*scope, |s| {
                    // todo this can panic
                    s.instructions.insert(*index, *new_instruction.clone());
                    Ok(())
                });
                if let Err(e) = updated {
                    return e.into();
                }
            }
            Instruction::UpdateInstruction(scope, index, new_instruction) => {
                let updated = self.update_scope(*scope, |s| {
                    match s.instructions.get_mut(*index) {
                        None => {
                            return Err(VMError::ScopeDoesNotExist(format!(
                                "Instruction does not exist: {}",
                                index
                            )))
                        }
                        Some(i) => {
                            *i = *new_instruction.clone();
                        }
                    }
                    Ok(())
                });
                if let Err(e) = updated {
                    return e.into();
                }
            }
            &Instruction::RemoveInstruction(scope, index) => {
                let updated = self.update_scope(scope, |s| {
                    if index >= s.instructions.len() {
                        return Err(VMError::UnsupportedOperation(format!(
                            "Instruction does not exist: {}#{}",
                            scope, index
                        )));
                    }
                    s.instructions.remove(index);
                    Ok(())
                });
                if let Err(e) = updated {
                    return e.into();
                }
            }
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
                    let s = self.pop();
                    if s.is_none() {
                        break;
                    }
                }
            }
            &Instruction::CallMemo(scope) => {
                if let Err(e) = self.call_frame_memo(scope) {
                    return e.into();
                }
            }
            &Instruction::ForList { scope } => {
                let result = self.call_for_comprehension(scope, Vec::with_capacity, |result, value| {
                    if !value.is_none() {
                        result.push(value);
                    }
                    None
                });
                match result {
                    Ok(r) => self.store_value(r.into()),
                    Err(e) => return e
                }
            }
            &Instruction::ForMap { scope } => {
                let result = self.call_for_comprehension(scope, IndexMap::with_capacity, |result, value| {
                    match value {
                        ObjectValue::Primitive(PrimitiveValue::None) => {}
                        ObjectValue::Tuple(mut t) if t.len() >= 2 => {
                            // todo this should be == 2 but same tuple is reused appending to front
                            let v = t.remove(1);
                            let k = t.remove(0);
                            if !k.is_none() && !v.is_none() {
                                result.insert(k, v);
                            }
                        }
                        // todo should a single value be both the key & value?
                        _ => {
                            let e: ObjectValue = VMError::UnsupportedOperation(format!(
                                "Invalid args in for-map {value}"
                            ))
                                .into();
                            result.insert(e.clone(), e);
                        }
                    }
                    None
                });
                match result {
                    Ok(r) => self.store_value(r.into()),
                    Err(e) => return e
                }
            }
            &Instruction::Send(args) => {
                if let Err(o) = self.send(args) {
                    return o.into();
                }
            }
            &Instruction::Spawn(scope_id, timeout) => {
                let timeout = if timeout {
                    let v = self.next_resolved_value(|| "spawn");
                    let v = v.borrow();
                    match v.to_usize() {
                        Ok(u) => Some(u),
                        Err(o) => return o.into(),
                    }
                } else {
                    None
                };
                if let Err(o) = self.spawn(scope_id, timeout) {
                    return o.into();
                }
            }
            &Instruction::Receive(args) => {
                if let Err(o) = self.receive(args) {
                    return o.into();
                }
            }
            Instruction::Sleep => {
                let v = self.next_resolved_value(|| "sleep");
                let duration = match v.borrow().to_usize() {
                    Ok(v) => Duration::from_millis(v as u64),
                    Err(e) => return e.into(),
                };
                self.sleep(duration);
                self.store_value(ObjectValue::default().into());
            }
            Instruction::CreateObject(ob) => {
                self.store_value(RigzObject::new(ob.clone()).into());
            }
            &Instruction::CreateDependency(args, dep) => {
                let args = self.resolve_args(args).into();
                let res = self
                    .call_dependency(args, dep, CallType::Create)
                    .unwrap_or_else(|e| e.into());
                self.store_value(res.into());
            }
            Instruction::CallObject { dep, func, args } => {
                let args = self.resolve_args(*args).into();
                let res = self
                    .call_dependency(args, *dep, CallType::Call(func))
                    .unwrap_or_else(|e| e.into());
                self.store_value(res.into());
            }
            Instruction::CallObjectExtension { func, args } => {
                let args = self.resolve_args(*args).into();
                let v = self.next_resolved_value(|| "object_extension");
                let v = match v.borrow().deref() {
                    ObjectValue::Object(o) => o.call_extension(func, args),
                    s => Err(VMError::UnsupportedOperation(format!(
                        "{s}.{func} is not callable"
                    ))),
                };
                self.store_value(v.into());
            }
            Instruction::CallMutableObjectExtension { func, args } => {
                let args = self.resolve_args(*args).into();
                let v = self.next_resolved_value(|| "mut_object_extension");
                let v = match v.borrow_mut().deref_mut() {
                    ObjectValue::Object(o) => o.call_mutable_extension(func, args),
                    s => Err(VMError::UnsupportedOperation(format!(
                        "{s}.{func} is not callable"
                    ))),
                };

                if let Ok(None) = v {
                } else {
                    self.store_value(v.into());
                }
            }
            Instruction::Try => {
                let next = self.next_resolved_value(|| "try");
                if next.borrow().is_error() {
                    return VMState::Ran(next);
                } else {
                    self.store_value(next.into())
                }
            }
            &Instruction::Catch(scope, has_arg) => {
                let next = self.next_resolved_value(|| "catch");
                if next.borrow().is_error() {
                    let ObjectValue::Primitive(PrimitiveValue::Error(e)) = next.take() else {
                        unreachable!("is_error check failed")
                    };
                    if has_arg {
                        let v = e.extract_value();
                        self.store_value(v.into());
                    }
                    if let Err(e) = self.call_frame(scope) {
                        self.store_value(e.into())
                    }
                } else {
                    self.store_value(next.into())
                }
            }
            &Instruction::CreateEnum {
                enum_type,
                variant,
                has_expression,
            } => {
                let decl = match self.find_enum(enum_type) {
                    Ok(v) => v,
                    Err(e) => return e.into(),
                };
                let value = if has_expression {
                    Some(self.next_resolved_value(|| "create_enum"))
                } else {
                    None
                };
                let name = match decl.variants.get(variant) {
                    None => {
                        return VMError::runtime(format!(
                            "Invalid enum variant {} for {}",
                            variant, decl.name
                        ))
                        .into()
                    }
                    Some((v, _)) => v.clone(),
                };
                let value = StackValue::Value(Rc::new(RefCell::new(ObjectValue::Enum(
                    enum_type,
                    variant,
                    value.map(|v| v.borrow().clone().into()),
                ))));
                self.store_value(value)
            }
            Instruction::Match(arms) => {
                let v = self.next_resolved_value(|| "match");
                let mut scope = None;
                for arm in arms {
                    match arm {
                        MatchArm::Enum(va, sc) => {
                            // todo support destructuring
                            if let ObjectValue::Enum(_, ev, val) = v.borrow().deref() {
                                // todo ensure enum types are the same
                                if ev == va {
                                    scope = Some(sc);
                                    break;
                                }
                            }
                        }
                        MatchArm::If(va, sc) => {}
                        MatchArm::Unless(va, sc) => {}
                        MatchArm::Else(s) => {
                            scope = Some(s);
                            break;
                        }
                    }
                }
                match scope {
                    None => {
                        return VMError::runtime("No value found for match expression".to_string())
                            .into()
                    }
                    Some(s) => match self.call_frame(*s) {
                        Ok(_) => {}
                        Err(e) => return e.into(),
                    },
                }
            }
            &Instruction::Loop(scope_id) => {
                if let Some(e) = self.call_loop(scope_id) {
                    return e;
                }
            }
            &Instruction::For { scope } => {
                if let Some(e) = self.call_for(scope) {
                    return e;
                }
            }
            Instruction::Break => return VMState::Break,
            Instruction::Next => return VMState::Next,
            ins => {
                return VMError::todo(format!("Instruction is not supported yet {ins:?}")).into()
            }
        };
        VMState::Running
    }

    #[inline]
    fn instance_get(&mut self, multiple: bool) {
        let attr = self.next_resolved_value(|| "instance_get - attr");
        let source = self.next_resolved_value(|| "instance_get - source");
        let v = match source.borrow().get(attr.borrow().deref()) {
            Ok(Some(v)) => v,
            Ok(None) => ObjectValue::default(),
            Err(e) => e.into(),
        };
        if multiple {
            self.store_value(source.into());
        }
        self.store_value(v.into());
    }

    #[inline]
    fn instance_set(&mut self, mutable: bool) {
        let value = self.next_resolved_value(|| "instance_set - value");
        let attr = self.next_resolved_value(|| "instance_set - attr");
        let source = self.next_resolved_value(|| "instance_set - source");
        let value = value.borrow();
        let source = if mutable {
            source
        } else {
            source.borrow().clone().into()
        };
        let s = { source.borrow_mut().instance_set(attr, value.deref()) };
        if let Err(e) = s {
            source.replace(e.into());
        }
        if !mutable {
            self.store_value(source.into());
        }
    }
}
