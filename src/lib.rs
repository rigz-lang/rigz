extern crate core;

mod builder;
mod value;
mod number;
mod macros;

use std::cell::LazyCell;
use std::fmt::{Debug};
use std::hash::{Hash, Hasher};
use std::string::ToString;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use crate::value::Value;

pub use builder::VMBuilder;

pub trait Rev {
    type Output;

    fn rev(self) -> Self::Output;
}

pub trait Logical<Rhs> {
    type Output;

    fn and(self, rhs: Rhs) -> Self::Output;
    fn or(self, rhs: Rhs) -> Self::Output;
    fn xor(self, rhs: Rhs) -> Self::Output;
}

pub type Register = usize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VMError {
    RuntimeError(String),
    FrameError(String),
    ScopeError(String),
    InvalidPC(String),
    EmptyRegister(String),
    ConversionError(String),
    ScopeDoesNotExist(String),
}

#[derive(Clone, Debug)]
pub enum UnaryOperation {
    Neg,
    Not,
    Rev,
    Print,
    EPrint,
}

#[derive(Clone, Debug)]
pub enum BinaryOperation {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shr,
    Shl,
    BitOr,
    BitAnd,
    BitXor,
    Or,
    And,
    Xor,
    Eq,
    Neq,
}

#[derive(Clone, Debug)]
pub enum Instruction<'vm> {
    Halt(Register),
    Unary {
        op: UnaryOperation,
        from: Register,
        output: Register,
    },
    Binary {
        op: BinaryOperation,
        lhs: Register,
        rhs: Register,
        output: Register,
    },
    Load(Register, Value<'vm>),
    Copy(Register, Register),
    Call(usize),
    Ret,
    Cast {
        from: Register,
        to: Register,
        rigz_type: RigzType
    },
    // Import(),
    // Export(),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RigzType {
    None,
    Bool,
    Int,
    Float,
    Number,
    UInt,
    String,
    List,
    Map,
    Error,
    Function(Vec<RigzType>, Box<RigzType>),
    Object(RigzObjectDefinition)
}


#[derive(Clone, Debug, PartialEq)]
pub struct RigzObjectDefinition {
    name: String,
    fields: IndexMap<String, RigzType>,
}

static NONE: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "None".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::None)]),
});

static BOOL: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "Bool".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::Bool)]),
});

static NUMBER: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "Number".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::Number)]),
});

static STRING: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "String".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::String)]),
});

static ERROR: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "Error".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::Error)]),
});

static LIST: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "List".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::List)]),
});

static MAP: Lazy<RigzObjectDefinition> = Lazy::new(|| RigzObjectDefinition {
    name: "Map".to_string(),
    fields: IndexMap::from([("value".to_string(), RigzType::Map)]),
});

#[derive(Clone, Debug, PartialEq)]
pub struct RigzObject<'vm> {
    fields: IndexMap<String, Value<'vm>>,
    definition_index: &'vm RigzObjectDefinition,
}

impl<'vm> RigzObject<'vm> {
    pub(crate) fn cast(&self, def: RigzObjectDefinition) -> Result<RigzObject<'vm>, VMError> {
        if self.definition_index == &def {
            return Ok(self.clone())
        }

        todo!()
    }
}

impl <'vm> RigzObject<'vm> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn equivalent(&self, other: &IndexMap<Value<'vm>, Value<'vm>>) -> bool {
        for (k, v) in other {
            let key = k.to_string();
            if !self.fields.contains_key(&key) {
                return false
            }
            match self.fields.get(&key) {
                None => return false,
                Some(o) => {
                    if !o.eq(v) {
                        return false
                    }
                }
            };
        }
        return true
    }
}

impl <'vm> Hash for RigzObject<'vm> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.definition_index.name.hash(state);
        for (k, v) in &self.fields {
            k.hash(state);
            v.hash(state);
        }
    }
}

#[derive(Clone, Debug)]
pub struct CallFrame<'vm> {
    scope_id: usize,
    pc: usize,
    variables: IndexMap<String, Value<'vm>>, // TODO switch to intern strings
    parent: Option<usize>
}

impl <'vm> Default for CallFrame<'vm> {
    fn default() -> Self {
        Self::main()
    }
}

impl <'vm> CallFrame<'vm> {
    fn next_instruction(&mut self, scope: &Scope<'vm>) -> Instruction<'vm> {
        let instruction = scope.instructions[self.pc].clone();
        self.pc += 1;
        instruction
    }

    pub fn main() -> Self {
        Self {
            scope_id: 0,
            pc: 0,
            variables: Default::default(),
            parent: None,
        }
    }

    pub fn child(scope_id: usize, parent: usize) -> Self {
        Self {
            scope_id,
            pc: 0,
            variables: Default::default(),
            parent: Some(parent),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Scope<'vm> {
    instructions: Vec<Instruction<'vm>>,
    type_definitions: IndexMap<String, RigzObjectDefinition>
}

impl <'vm> Scope<'vm> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug)]
pub struct Lifecycle<'vm> {
    name: String,
    parent: Option<&'vm Lifecycle<'vm>>
}

#[derive(Clone, Debug)]
pub struct VM<'vm> {
    scopes: Vec<Scope<'vm>>,
    current: CallFrame<'vm>,
    frames: Vec<CallFrame<'vm>>,
    registers: IndexMap<usize, Value<'vm>>,
    lifecycles: Vec<Lifecycle<'vm>>
}

impl <'vm> VM<'vm> {
    pub fn insert_register(&mut self, register: Register, value: Value<'vm>) {
        if register == 0 {
            return
        }

        self.registers.insert(register, value);
    }

    pub fn remove_register(&mut self, register: &Register) -> Result<Value<'vm>, VMError> {
        if *register == 0 {
            return Ok(Value::None)
        }

        match self.registers.shift_remove(register) {
            None => Err(VMError::EmptyRegister(format!("R{} is empty", register))),
            Some(v) => Ok(v),
        }
    }

    pub fn run(&mut self) -> Result<Value, VMError> {
        loop {
            let scope = self.current.scope_id;
            let scope = match self.scopes.get(scope) {
                None => return Err(VMError::ScopeError(format!("Scope {} does not exist", scope))),
                Some(s) => s.clone(),
            };
            let len = scope.instructions.len();
            if self.current.pc >= len {
                // TODO this should probably be an error requiring explicit halt, halt 0 returns none
                break;
            }

            let instruction = self.current.next_instruction(&scope);
            match instruction {
                Instruction::Halt(r) => {
                    return self.remove_register(&r)
                }
                Instruction::Unary { op, from, output } => {
                    let val = match self.registers.shift_remove(&from) {
                        None => return Err(VMError::EmptyRegister(format!("R{} is empty", from))),
                        Some(v) => v,
                    };
                    match op {
                        UnaryOperation::Neg => {
                            self.insert_register(output, val);
                        }
                        UnaryOperation::Not => {
                            self.insert_register(output, val);
                        },
                        UnaryOperation::Print => {
                            println!("{}", val);
                            self.insert_register(output, val);
                        }
                        UnaryOperation::EPrint => {
                            eprintln!("{}", val);
                            self.insert_register(output, val);
                        }
                        UnaryOperation::Rev => {
                            self.insert_register(output, val.rev());
                        }
                    }
                }
                Instruction::Binary { op, lhs, rhs, output } => {
                    let lhs = self.remove_register(&lhs)?;
                    let rhs = self.remove_register(&rhs)?;
                    let v = match op {
                        BinaryOperation::Add => lhs + rhs,
                        BinaryOperation::Sub => lhs - rhs,
                        BinaryOperation::Shr => lhs >> rhs,
                        BinaryOperation::Shl => lhs << rhs,
                        BinaryOperation::Eq => Value::Bool(lhs == rhs),
                        BinaryOperation::Neq => Value::Bool(lhs != rhs),
                        BinaryOperation::Mul => lhs * rhs,
                        BinaryOperation::Div => lhs / rhs,
                        BinaryOperation::Rem => lhs % rhs,
                        BinaryOperation::BitOr => lhs | rhs,
                        BinaryOperation::BitAnd => lhs & rhs,
                        BinaryOperation::BitXor => lhs ^ rhs,
                        BinaryOperation::And => lhs.and(rhs),
                        BinaryOperation::Or => lhs.or(rhs),
                        BinaryOperation::Xor => lhs.xor(rhs)
                    };

                    self.insert_register(output, v);
                }
                Instruction::Load(r, v) => {
                    self.insert_register(r, v);
                }
                Instruction::Copy(from, to) => {
                    let copy = match self.registers.get(&from) {
                        None => return Err(VMError::EmptyRegister(format!("R{} is empty", from))),
                        Some(s) => s.clone()
                    };
                    self.insert_register(to, copy);
                }
                Instruction::Call(scope_index) => {
                    if self.scopes.len() <= scope_index {
                        return Err(VMError::ScopeDoesNotExist(format!("{} does not exist", scope_index)))
                    }
                    let current = std::mem::take(&mut self.current);
                    self.frames.push(current);
                    self.current = CallFrame::child(scope_index, self.frames.len() - 1);
                }
                Instruction::Ret => {
                    match self.frames.pop() {
                        None => {
                            return Err(VMError::FrameError("CallStack is empty".to_string()))
                        }
                        Some(c) => {
                            self.current = c;
                        }
                    }
                }
                Instruction::Cast { from, rigz_type, to } => {
                    let value = self.remove_register(&from)?;
                    self.insert_register(to, value.cast(rigz_type)?);
                }
            }
        }
        Ok(Value::None)
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::{RigzType, VMBuilder};
    use crate::number::Number;
    use crate::value::Value;
    

    #[test]
    fn value_eq() {
        assert_eq!(Value::None, Value::None);
        assert_eq!(Value::None, Value::Bool(false));
        assert_eq!(Value::None, Value::Number(Number::Int(0)));
        assert_eq!(Value::None, Value::Number(Number::Float(0.0)));
        assert_eq!(Value::None, Value::Number(Number::UInt(0)));
        assert_eq!(Value::None, Value::String(String::new()));
        assert_eq!(Value::Bool(false), Value::String(String::new()));
        assert_eq!(Value::Number(Number::UInt(0)), Value::String(String::new()));
    }

    #[test]
    fn load_works() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(4, Value::Number(Number::Int(42)))
            .build();
        vm.run().unwrap();
        assert_eq!(vm.registers.get(&4).unwrap().clone(), Value::Number(Number::Int(42)));
    }

    #[test]
    fn cast_works() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(4, Value::Number(Number::Int(42)))
            .add_cast_instruction(4, RigzType::String, 7)
            .build();
        vm.run().unwrap();
        assert_eq!(vm.registers.get(&4), None);
        assert_eq!(vm.registers.get(&7).unwrap().clone(), Value::String(42.to_string()));
    }

    #[test]
    fn add_works() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(4, Value::Number(Number::Int(42)))
            .add_copy_instruction(4, 37)
            .add_add_instruction(4, 37, 82)
            .build();
        vm.run().unwrap();
        assert_eq!(vm.registers.get(&82).unwrap().clone(), Value::Number(Number::Int(84)));
    }

    #[test]
    fn copy_works() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(4, Value::Number(Number::Int(42)))
            .add_copy_instruction(4, 37)
            .build();
        vm.run().unwrap();
        assert_eq!(vm.registers.get(&37).unwrap().clone(), Value::Number(Number::Int(42)));
    }

    #[test]
    fn shr_works_str_number() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(1, Value::String(String::from_str("abc").unwrap()))
            .add_load_instruction(2, Value::Number(Number::Int(1)))
            .add_shr_instruction(1, 2, 3)
            .build();
        vm.run().unwrap();
        assert_eq!(vm.registers.get(&3).unwrap().clone(), Value::String(String::from_str("ab").unwrap()));
    }

    #[test]
    fn shl_works_str_number() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(1, Value::String(String::from_str("abc").unwrap()))
            .add_load_instruction(2, Value::Number(Number::Int(1)))
            .add_shl_instruction(1, 2, 3)
            .build();
        vm.run().unwrap();
        assert_eq!(vm.registers.get(&3).unwrap().clone(), Value::String(String::from_str("bc").unwrap()));
    }

    #[test]
    fn call_works() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(1, Value::String(String::from_str("abc").unwrap()))
            .enter_scope()
            .add_copy_instruction(1, 3)
            .exit_scope()
            .add_call_instruction(1)
            .build();
        vm.run().unwrap();
        assert_eq!(vm.registers.get(&3).unwrap().clone(), Value::String(String::from_str("abc").unwrap()));
    }
}
