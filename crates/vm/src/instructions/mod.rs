mod runner;

use crate::objects::RigzType;
use crate::vm::StackValue;
use crate::{BinaryOperation, UnaryOperation, Value};
use log::Level;
pub use runner::{ResolvedModule, Runner};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VMCallSite<'vm> {
    Scope(usize),
    Module { module: &'vm str, func: &'vm str },
    VMModule { module: &'vm str, func: &'vm str },
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
pub enum Instruction<'vm> {
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
    CallMatchingSelf(Vec<(VMArg, Vec<VMArg>, VMCallSite<'vm>)>),
    CallMatchingSelfMemo(Vec<(VMArg, Vec<VMArg>, VMCallSite<'vm>)>),
    CallMatching(Vec<(Vec<VMArg>, VMCallSite<'vm>)>),
    CallMatchingMemo(Vec<(Vec<VMArg>, VMCallSite<'vm>)>),
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
    Sleep,
    Send(usize),
    Broadcast(BroadcastArgs),
    Spawn(usize, bool),
    Receive(usize),
    /// Danger Zone, use these instructions at your own risk (sorted by risk)
    /// in the right situations these will be fantastic, otherwise avoid them
    Pop(usize),
    Goto(usize, usize),
    AddInstruction(usize, Box<Instruction<'vm>>),
    InsertAtInstruction(usize, usize, Box<Instruction<'vm>>),
    UpdateInstruction(usize, usize, Box<Instruction<'vm>>),
    RemoveInstruction(usize, usize),
}
