mod runner;

use crate::objects::RigzType;
use crate::process::Reference;
use crate::vm::StackValue;
use crate::{BinaryOperation, Snapshot, UnaryOperation, VMError, Value};
use log::Level;
pub use runner::{ResolvedModule, Runner, THIS_VAR};
use std::fmt::Display;
use std::vec::IntoIter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VMCallSite {
    Scope(usize),
    Module {
        module: Reference<String>,
        func: Reference<String>,
    },
    VMModule {
        module: Reference<String>,
        func: Reference<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VMArg {
    Type(RigzType),
    Value(Value),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadValue {
    ScopeId(usize),
    Value(Value),
    Constant(usize),
}

impl Snapshot for LoadValue {
    fn as_bytes(&self) -> Vec<u8> {
        let mut results = Vec::new();
        match self {
            LoadValue::ScopeId(s) => {
                results.push(0);
                results.extend(s.as_bytes());
            }
            LoadValue::Value(v) => {
                results.push(1);
                results.extend(v.as_bytes());
            }
            LoadValue::Constant(c) => {
                results.push(2);
                results.extend(c.as_bytes());
            }
        }
        results
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let tv = match bytes.next() {
            None => return Err(VMError::RuntimeError(format!("{location} LoadValue type"))),
            Some(b) => b,
        };
        let l = match tv {
            0 => LoadValue::ScopeId(Snapshot::from_bytes(bytes, location)?),
            1 => LoadValue::Value(Snapshot::from_bytes(bytes, location)?),
            2 => LoadValue::Constant(Snapshot::from_bytes(bytes, location)?),
            _ => {
                return Err(VMError::RuntimeError(format!(
                    "{location} Invalid LoadValue type {tv}"
                )))
            }
        };
        Ok(l)
    }
}

impl<T: Into<Value>> From<T> for LoadValue {
    #[inline]
    fn from(value: T) -> Self {
        LoadValue::Value(value.into())
    }
}

impl From<LoadValue> for StackValue {
    #[inline]
    fn from(value: LoadValue) -> Self {
        match value {
            LoadValue::ScopeId(s) => StackValue::ScopeId(s),
            LoadValue::Value(v) => StackValue::Value(v.into()),
            LoadValue::Constant(c) => StackValue::Constant(c),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BroadcastArgs {
    Args(usize),
    All(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Halt,
    HaltIfError,
    Unary(UnaryOperation),
    Binary(BinaryOperation),
    BinaryAssign(BinaryOperation),
    Load(LoadValue),
    InstanceGet(bool),
    InstanceSet,
    InstanceSetMut,
    Call(usize),
    CallMemo(usize),
    CallMatchingSelf(Vec<(VMArg, Vec<VMArg>, VMCallSite)>),
    CallMatchingSelfMemo(Vec<(VMArg, Vec<VMArg>, VMCallSite)>),
    CallMatching(Vec<(Vec<VMArg>, VMCallSite)>),
    CallMatchingMemo(Vec<(Vec<VMArg>, VMCallSite)>),
    Log(Level, String, usize),
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
    GetVariable(Reference<String>),
    GetMutableVariable(Reference<String>),
    GetVariableReference(Reference<String>),
    LoadLet(Reference<String>),
    LoadMut(Reference<String>),
    PersistScope(Reference<String>),
    // requires modules, enabled by default
    /// Module instructions will clone your module, ideally modules implement Copy + Clone
    CallModule {
        module: Reference<String>,
        func: Reference<String>,
        args: usize,
    },
    CallExtension {
        module: Reference<String>,
        func: Reference<String>,
        args: usize,
    },
    CallMutableExtension {
        module: Reference<String>,
        func: Reference<String>,
        args: usize,
    },
    CallVMExtension {
        module: Reference<String>,
        func: Reference<String>,
        args: usize,
    },
    ForList {
        scope: usize,
    },
    ForMap {
        scope: usize,
    },
    Sleep,
    Send(usize),
    Broadcast(BroadcastArgs),
    Spawn(usize, bool),
    Receive(usize),
    /// Danger Zone, use these instructions at your own risk (sorted by risk)
    /// in the right situations these will be fantastic, otherwise avoid them
    Pop(usize),
    Goto(usize, usize),
    AddInstruction(usize, Box<Instruction>),
    InsertAtInstruction(usize, usize, Box<Instruction>),
    UpdateInstruction(usize, usize, Box<Instruction>),
    RemoveInstruction(usize, usize),
}

impl Snapshot for Instruction {
    fn as_bytes(&self) -> Vec<u8> {
        todo!()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        todo!()
    }
}
