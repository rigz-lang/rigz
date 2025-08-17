mod runner;

use log::Level;
use rigz_core::{BinaryAssignOperation, BinaryOperation, ObjectValue, RigzType, Snapshot, StackValue, UnaryOperation, VMError};
pub use runner::{CallType, ResolvedModule, Runner};
use std::fmt::Display;
use std::sync::Arc;
use std::vec::IntoIter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VMCallSite {
    Scope(usize),
    Module { module: usize, func: String },
    VMModule { module: usize, func: String },
}

impl Snapshot for VMCallSite {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            VMCallSite::Scope(i) => {
                let mut res = vec![0];
                res.extend(i.as_bytes());
                res
            }
            VMCallSite::Module { module, func } => {
                let mut res = vec![1];
                res.extend(module.as_bytes());
                res.extend(func.as_bytes());
                res
            }
            VMCallSite::VMModule { module, func } => {
                let mut res = vec![2];
                res.extend(module.as_bytes());
                res.extend(func.as_bytes());
                res
            }
        }
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            Some(b) => b,
            None => {
                return Err(VMError::runtime(format!(
                    "Missing VMCallSite byte {location}"
                )))
            }
        };

        let v = match next {
            0 => VMCallSite::Scope(Snapshot::from_bytes(bytes, location)?),
            1 => VMCallSite::Module {
                module: Snapshot::from_bytes(bytes, location)?,
                func: Snapshot::from_bytes(bytes, location)?,
            },
            2 => VMCallSite::VMModule {
                module: Snapshot::from_bytes(bytes, location)?,
                func: Snapshot::from_bytes(bytes, location)?,
            },
            b => {
                return Err(VMError::runtime(format!(
                    "Invalid VMCallSite byte {b} - {location}"
                )))
            }
        };
        Ok(v)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VMArg {
    Type(RigzType),
    Value(ObjectValue),
}

impl Snapshot for VMArg {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            VMArg::Type(rt) => {
                let mut res = vec![0];
                res.extend(rt.as_bytes());
                res
            }
            VMArg::Value(v) => {
                let mut res = vec![1];
                res.extend(v.as_bytes());
                res
            }
        }
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            Some(b) => b,
            None => return Err(VMError::runtime(format!("Missing VMArg byte {location}"))),
        };

        let v = match next {
            0 => VMArg::Type(Snapshot::from_bytes(bytes, location)?),
            1 => VMArg::Value(Snapshot::from_bytes(bytes, location)?),
            b => {
                return Err(VMError::runtime(format!(
                    "Invalid VMArg byte {b} - {location}"
                )))
            }
        };
        Ok(v)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadValue {
    ScopeId(usize),
    Value(ObjectValue),
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
            None => return Err(VMError::runtime(format!("{location} LoadValue type"))),
            Some(b) => b,
        };
        let l = match tv {
            0 => LoadValue::ScopeId(Snapshot::from_bytes(bytes, location)?),
            1 => LoadValue::Value(Snapshot::from_bytes(bytes, location)?),
            2 => LoadValue::Constant(Snapshot::from_bytes(bytes, location)?),
            _ => {
                return Err(VMError::runtime(format!(
                    "{location} Invalid LoadValue type {tv}"
                )))
            }
        };
        Ok(l)
    }
}

impl<T: Into<ObjectValue>> From<T> for LoadValue {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatchArm {
    Enum(usize, usize),
    If(usize, usize),
    Unless(usize, usize),
    Else(usize),
}

impl Snapshot for MatchArm {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            MatchArm::Enum(c, v) => {
                let mut res = vec![0];
                res.extend(c.as_bytes());
                res.extend(v.as_bytes());
                res
            }
            MatchArm::If(c, v) => {
                let mut res = vec![1];
                res.extend(c.as_bytes());
                res.extend(v.as_bytes());
                res
            }
            MatchArm::Unless(c, v) => {
                let mut res = vec![2];
                res.extend(c.as_bytes());
                res.extend(v.as_bytes());
                res
            }
            MatchArm::Else(v) => {
                let mut res = vec![3];
                res.extend(v.as_bytes());
                res
            }
        }
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let current = match bytes.next() {
            None => {
                return Err(VMError::runtime(format!(
                    "Missing match arm byte {location}"
                )))
            }
            Some(b) => b,
        };
        let arm = match current {
            0 => MatchArm::Enum(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            1 => MatchArm::If(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            2 => MatchArm::Unless(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            3 => MatchArm::Else(Snapshot::from_bytes(bytes, location)?),
            b => {
                return Err(VMError::runtime(format!(
                    "Illegal match arm byte {b} {location}"
                )))
            }
        };
        Ok(arm)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DisplayType {
    Puts,
    EPrint,
    EPrintLn,
    Print,
    PrintLn,
}

impl Snapshot for DisplayType {
    fn as_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            None => {
                return Err(VMError::runtime(format!(
                    "Missing DisplayType byte {location}"
                )))
            }
            Some(b) => b,
        };

        let op = match next {
            0 => DisplayType::Print,
            1 => DisplayType::EPrint,
            2 => DisplayType::PrintLn,
            3 => DisplayType::EPrintLn,
            b => {
                return Err(VMError::runtime(format!(
                    "Illegal DisplayType byte {b} - {location}"
                )))
            }
        };
        Ok(op)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Halt,
    HaltIfError,
    Unary(UnaryOperation),
    Binary(BinaryOperation),
    BinaryAssign(BinaryAssignOperation),
    Load(LoadValue),
    InstanceGet(bool),
    InstanceSet,
    InstanceSetMut,
    Call(usize),
    CreateDependency(usize, usize),
    CallMemo(usize),
    CallMatchingSelf(Vec<(VMArg, Vec<VMArg>, VMCallSite)>),
    CallMatchingSelfMemo(Vec<(VMArg, Vec<VMArg>, VMCallSite)>),
    CallMatching(Vec<(Vec<VMArg>, VMCallSite)>),
    CallMatchingMemo(Vec<(Vec<VMArg>, VMCallSite)>),
    CreateObject(Arc<RigzType>, usize),
    CreateEnum {
        enum_type: usize,
        variant: usize,
        has_expression: bool,
    },
    Match(Vec<MatchArm>),
    Log(Level, String, usize),
    Display(usize, DisplayType),
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
    GetVariable(usize),
    GetMutableVariable(usize),
    GetVariableReference(usize),
    LoadLet(usize, bool),
    LoadMut(usize, bool),
    PersistScope(usize),
    // requires modules, enabled by default
    /// Module instructions will clone your module, ideally modules implement Copy + Clone
    CallModule {
        module: usize,
        func: String,
        args: usize,
    },
    CallExtension {
        module: usize,
        func: String,
        args: usize,
    },
    CallMutableExtension {
        module: usize,
        func: String,
        args: usize,
    },
    CallObject {
        dep: usize,
        func: String,
        args: usize,
    },
    CallObjectExtension {
        func: String,
        args: usize,
    },
    CallMutableObjectExtension {
        func: String,
        args: usize,
    },
    // CallVMExtension {
    //     module: String,
    //     func: String,
    //     args: usize,
    // },
    ForList {
        scope: usize,
    },
    ForMap {
        scope: usize,
    },
    For {
        scope: usize,
    },
    Sleep,
    Send(usize),
    Spawn(usize, bool),
    Receive(usize),
    Try,
    Catch(usize, bool),
    Break,
    Next,
    Loop(usize),
    Exit,
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
        match self {
            Instruction::Halt => vec![0],
            Instruction::HaltIfError => vec![1],
            Instruction::Unary(a) => {
                let mut res = vec![2];
                res.extend(a.as_bytes());
                res
            }
            Instruction::Binary(a) => {
                let mut res = vec![3];
                res.extend(a.as_bytes());
                res
            }
            Instruction::BinaryAssign(a) => {
                let mut res = vec![4];
                res.extend(a.as_bytes());
                res
            }
            Instruction::Load(v) => {
                let mut res = vec![5];
                res.extend(v.as_bytes());
                res
            }
            Instruction::InstanceGet(v) => {
                let mut res = vec![6];
                res.extend(v.as_bytes());
                res
            }
            Instruction::InstanceSet => vec![7],
            Instruction::InstanceSetMut => vec![8],
            Instruction::Call(s) => {
                let mut res = vec![9];
                res.extend(s.as_bytes());
                res
            }
            Instruction::CallMemo(s) => {
                let mut res = vec![10];
                res.extend(s.as_bytes());
                res
            }
            Instruction::CallMatchingSelf(s) => {
                let mut res = vec![11];
                res.extend(s.as_bytes());
                res
            }
            Instruction::CallMatchingSelfMemo(s) => {
                let mut res = vec![12];
                res.extend(s.as_bytes());
                res
            }
            Instruction::CallMatching(s) => {
                let mut res = vec![13];
                res.extend(s.as_bytes());
                res
            }
            Instruction::CallMatchingMemo(s) => {
                let mut res = vec![14];
                res.extend(s.as_bytes());
                res
            }
            Instruction::Log(l, t, a) => {
                let mut res = vec![15];
                res.extend(l.as_bytes());
                res.extend(t.as_bytes());
                res.extend(a.as_bytes());
                res
            }
            Instruction::Display(args, ty) => {
                let mut res = vec![16];
                res.extend(args.as_bytes());
                res.extend(ty.as_bytes());
                res
            }
            Instruction::CallEq(a) => {
                let mut res = vec![17];
                res.extend(a.as_bytes());
                res
            }
            Instruction::CallNeq(a) => {
                let mut res = vec![18];
                res.extend(a.as_bytes());
                res
            }
            Instruction::IfElse {
                if_scope,
                else_scope,
            } => {
                let mut res = vec![19];
                res.extend(if_scope.as_bytes());
                res.extend(else_scope.as_bytes());
                res
            }
            Instruction::If(i) => {
                let mut res = vec![20];
                res.extend(i.as_bytes());
                res
            }
            Instruction::Unless(u) => {
                let mut res = vec![21];
                res.extend(u.as_bytes());
                res
            }
            Instruction::Cast { rigz_type } => {
                let mut res = vec![22];
                res.extend(rigz_type.as_bytes());
                res
            }
            Instruction::Ret => vec![23],
            Instruction::GetVariable(v) => {
                let mut res = vec![24];
                res.extend(v.as_bytes());
                res
            }
            Instruction::GetMutableVariable(v) => {
                let mut res = vec![25];
                res.extend(v.as_bytes());
                res
            }
            Instruction::GetVariableReference(v) => {
                let mut res = vec![26];
                res.extend(v.as_bytes());
                res
            }
            Instruction::LoadLet(v, shadow) => {
                let mut res = vec![27];
                res.extend(v.as_bytes());
                res.extend(shadow.as_bytes());
                res
            }
            Instruction::LoadMut(v, shadow) => {
                let mut res = vec![28];
                res.extend(v.as_bytes());
                res.extend(shadow.as_bytes());
                res
            }
            Instruction::PersistScope(v) => {
                let mut res = vec![29];
                res.extend(v.as_bytes());
                res
            }
            Instruction::CallModule { module, func, args } => {
                let mut res = vec![30];
                res.extend(Snapshot::as_bytes(module));
                res.extend(Snapshot::as_bytes(func));
                res.extend(args.as_bytes());
                res
            }
            Instruction::CallExtension { module, func, args } => {
                let mut res = vec![31];
                res.extend(Snapshot::as_bytes(module));
                res.extend(Snapshot::as_bytes(func));
                res.extend(args.as_bytes());
                res
            }
            Instruction::CallMutableExtension { module, func, args } => {
                let mut res = vec![32];
                res.extend(Snapshot::as_bytes(module));
                res.extend(Snapshot::as_bytes(func));
                res.extend(args.as_bytes());
                res
            }
            // Instruction::CallVMExtension { module, func, args } => {
            //     let mut res = vec![33];
            //     res.extend(Snapshot::as_bytes(module));
            //     res.extend(Snapshot::as_bytes(func));
            //     res.extend(args.as_bytes());
            //     res
            // }
            Instruction::ForList { scope } => {
                let mut res = vec![34];
                res.extend(scope.as_bytes());
                res
            }
            Instruction::ForMap { scope } => {
                let mut res = vec![35];
                res.extend(scope.as_bytes());
                res
            }
            Instruction::Sleep => vec![36],
            Instruction::Send(v) => {
                let mut res = vec![37];
                res.extend(v.as_bytes());
                res
            }
            Instruction::Spawn(a, b) => {
                let mut res = vec![38];
                res.extend(a.as_bytes());
                res.extend(b.as_bytes());
                res
            }
            Instruction::Receive(v) => {
                let mut res = vec![39];
                res.extend(v.as_bytes());
                res
            }
            Instruction::Pop(v) => {
                let mut res = vec![40];
                res.extend(v.as_bytes());
                res
            }
            Instruction::Goto(s, v) => {
                let mut res = vec![41];
                res.extend(s.as_bytes());
                res.extend(v.as_bytes());
                res
            }
            Instruction::AddInstruction(s, i) => {
                let mut res = vec![42];
                res.extend(s.as_bytes());
                res.extend(i.as_bytes());
                res
            }
            Instruction::InsertAtInstruction(s, v, i) => {
                let mut res = vec![43];
                res.extend(s.as_bytes());
                res.extend(v.as_bytes());
                res.extend(i.as_bytes());
                res
            }
            Instruction::UpdateInstruction(s, v, i) => {
                let mut res = vec![44];
                res.extend(s.as_bytes());
                res.extend(v.as_bytes());
                res.extend(i.as_bytes());
                res
            }
            Instruction::RemoveInstruction(s, v) => {
                let mut res = vec![45];
                res.extend(s.as_bytes());
                res.extend(v.as_bytes());
                res
            }
            Instruction::CreateObject(o, args) => {
                let mut res = vec![46];
                res.extend(o.as_bytes());
                res.extend(args.as_bytes());
                res
            }
            Instruction::CreateDependency(args, dep) => {
                let mut res = vec![47];
                res.extend(args.as_bytes());
                res.extend(dep.as_bytes());
                res
            }
            Instruction::CallObject { dep, func, args } => {
                let mut res = vec![48];
                res.extend(dep.as_bytes());
                res.extend(func.as_bytes());
                res.extend(args.as_bytes());
                res
            }
            Instruction::CallObjectExtension { func, args } => {
                let mut res = vec![49];
                res.extend(func.as_bytes());
                res.extend(args.as_bytes());
                res
            }
            Instruction::CallMutableObjectExtension { func, args } => {
                let mut res = vec![50];
                res.extend(func.as_bytes());
                res.extend(args.as_bytes());
                res
            }
            Instruction::Try => vec![51],
            Instruction::Catch(scope, has_arg) => {
                let mut res = vec![52];
                res.extend(scope.as_bytes());
                res.extend(has_arg.as_bytes());
                res
            }
            Instruction::CreateEnum {
                enum_type,
                variant,
                has_expression,
            } => {
                let mut res = vec![53];
                res.extend(enum_type.as_bytes());
                res.extend(variant.as_bytes());
                res.extend(has_expression.as_bytes());
                res
            }
            Instruction::Match(arms) => {
                let mut res = vec![54];
                res.extend(arms.as_bytes());
                res
            }
            Instruction::Break => {
                vec![55]
            }
            Instruction::Loop(scope) => {
                let mut res = vec![56];
                res.extend(scope.as_bytes());
                res
            }
            Instruction::Next => {
                vec![57]
            }
            Instruction::For { scope } => {
                let mut res = vec![58];
                res.extend(scope.as_bytes());
                res
            }
            Instruction::Exit => {
                vec![59]
            }
        }
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let current = match bytes.next() {
            None => {
                return Err(VMError::runtime(format!(
                    "Missing instruction byte {location}"
                )))
            }
            Some(b) => b,
        };
        let ins = match current {
            0 => Instruction::Halt,
            1 => Instruction::HaltIfError,
            2 => Instruction::Unary(Snapshot::from_bytes(bytes, location)?),
            3 => Instruction::Binary(Snapshot::from_bytes(bytes, location)?),
            4 => Instruction::BinaryAssign(Snapshot::from_bytes(bytes, location)?),
            5 => Instruction::Load(Snapshot::from_bytes(bytes, location)?),
            6 => Instruction::InstanceGet(Snapshot::from_bytes(bytes, location)?),
            7 => Instruction::InstanceSet,
            8 => Instruction::InstanceSetMut,
            9 => Instruction::Call(Snapshot::from_bytes(bytes, location)?),
            10 => Instruction::CallMemo(Snapshot::from_bytes(bytes, location)?),
            11 => Instruction::CallMatchingSelf(Snapshot::from_bytes(bytes, location)?),
            12 => Instruction::CallMatchingSelfMemo(Snapshot::from_bytes(bytes, location)?),
            13 => Instruction::CallMatching(Snapshot::from_bytes(bytes, location)?),
            14 => Instruction::CallMatchingMemo(Snapshot::from_bytes(bytes, location)?),
            15 => Instruction::Log(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            16 => Instruction::Display(Snapshot::from_bytes(bytes, location)?, Snapshot::from_bytes(bytes, location)?),
            17 => Instruction::CallEq(Snapshot::from_bytes(bytes, location)?),
            18 => Instruction::CallNeq(Snapshot::from_bytes(bytes, location)?),
            19 => Instruction::IfElse {
                if_scope: Snapshot::from_bytes(bytes, location)?,
                else_scope: Snapshot::from_bytes(bytes, location)?,
            },
            20 => Instruction::If(Snapshot::from_bytes(bytes, location)?),
            21 => Instruction::Unless(Snapshot::from_bytes(bytes, location)?),
            22 => Instruction::Cast {
                rigz_type: Snapshot::from_bytes(bytes, location)?,
            },
            23 => Instruction::Ret,
            24 => Instruction::GetVariable(Snapshot::from_bytes(bytes, location)?),
            25 => Instruction::GetMutableVariable(Snapshot::from_bytes(bytes, location)?),
            26 => Instruction::GetVariableReference(Snapshot::from_bytes(bytes, location)?),
            27 => Instruction::LoadLet(
                Snapshot::from_bytes(bytes, location)?,
                bool::from_bytes(bytes, location)?,
            ),
            28 => Instruction::LoadMut(
                Snapshot::from_bytes(bytes, location)?,
                bool::from_bytes(bytes, location)?,
            ),
            29 => Instruction::PersistScope(Snapshot::from_bytes(bytes, location)?),
            30 => Instruction::CallModule {
                module: Snapshot::from_bytes(bytes, location)?,
                func: Snapshot::from_bytes(bytes, location)?,
                args: Snapshot::from_bytes(bytes, location)?,
            },
            31 => Instruction::CallExtension {
                module: Snapshot::from_bytes(bytes, location)?,
                func: Snapshot::from_bytes(bytes, location)?,
                args: Snapshot::from_bytes(bytes, location)?,
            },
            32 => Instruction::CallMutableExtension {
                module: Snapshot::from_bytes(bytes, location)?,
                func: Snapshot::from_bytes(bytes, location)?,
                args: Snapshot::from_bytes(bytes, location)?,
            },
            // 33 => Instruction::CallVMExtension {
            //     module: Snapshot::from_bytes(bytes, location)?,
            //     func: Snapshot::from_bytes(bytes, location)?,
            //     args: Snapshot::from_bytes(bytes, location)?,
            // },
            34 => Instruction::ForList {
                scope: Snapshot::from_bytes(bytes, location)?,
            },
            35 => Instruction::ForMap {
                scope: Snapshot::from_bytes(bytes, location)?,
            },
            36 => Instruction::Sleep,
            37 => Instruction::Send(Snapshot::from_bytes(bytes, location)?),
            38 => Instruction::Spawn(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            39 => Instruction::Receive(Snapshot::from_bytes(bytes, location)?),
            40 => Instruction::Pop(Snapshot::from_bytes(bytes, location)?),
            41 => Instruction::Goto(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            42 => Instruction::AddInstruction(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            43 => Instruction::InsertAtInstruction(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            44 => Instruction::UpdateInstruction(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            45 => Instruction::RemoveInstruction(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            46 => Instruction::CreateObject(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            47 => Instruction::CreateDependency(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            48 => Instruction::CallObject {
                dep: Snapshot::from_bytes(bytes, location)?,
                func: Snapshot::from_bytes(bytes, location)?,
                args: Snapshot::from_bytes(bytes, location)?,
            },
            49 => Instruction::CallObjectExtension {
                func: Snapshot::from_bytes(bytes, location)?,
                args: Snapshot::from_bytes(bytes, location)?,
            },
            50 => Instruction::CallMutableObjectExtension {
                func: Snapshot::from_bytes(bytes, location)?,
                args: Snapshot::from_bytes(bytes, location)?,
            },
            51 => Instruction::Try,
            52 => Instruction::Catch(
                Snapshot::from_bytes(bytes, location)?,
                Snapshot::from_bytes(bytes, location)?,
            ),
            53 => Instruction::CreateEnum {
                enum_type: Snapshot::from_bytes(bytes, location)?,
                variant: Snapshot::from_bytes(bytes, location)?,
                has_expression: Snapshot::from_bytes(bytes, location)?,
            },
            54 => Instruction::Match(Snapshot::from_bytes(bytes, location)?),
            55 => Instruction::Break,
            56 => Instruction::Loop(Snapshot::from_bytes(bytes, location)?),
            57 => Instruction::Next,
            58 => Instruction::For {
                scope: Snapshot::from_bytes(bytes, location)?,
            },
            59 => Instruction::Exit,
            b => {
                return Err(VMError::runtime(format!(
                    "Illegal instruction byte {b} {location}"
                )))
            }
        };
        Ok(ins)
    }
}
