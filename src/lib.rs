extern crate core;

mod builder;
mod value;
mod number;

use std::fmt::{Debug};
use std::hash::Hash;
use indexmap::IndexMap;
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
pub enum Instruction {
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
    Load(Register, Value),
    Copy(Register, Register),
    Call(usize),
    Ret,
}

#[derive(Clone, Debug)]
pub struct CallFrame {
    scope_id: usize,
    pc: usize,
    variables: IndexMap<String, Value>, // TODO switch to intern strings
    parent: Option<usize>
}

impl CallFrame {
    fn next_instruction(&mut self, scope: &Scope) -> Instruction {
        let instruction = scope.instructions[self.pc].clone();
        self.pc += 1;
        instruction
    }
}

impl CallFrame {
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
pub struct Scope {
    instructions: Vec<Instruction>
}

impl Scope {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug)]
pub struct VM {
    scopes: Vec<Scope>,
    current: CallFrame,
    frames: Vec<CallFrame>,
    registers: IndexMap<usize, Value>,
}

impl VM {
    pub fn insert_register(&mut self, register: Register, value: Value) {
        if register == 0 {
            return
        }

        self.registers.insert(register, value);
    }

    pub fn remove_register(&mut self, register: &Register) -> Result<Value, VMError> {
        if *register == 0 {
            return Ok(Value::None)
        }

        match self.registers.shift_remove(register) {
            None => Err(VMError::EmptyRegister(format!("R{} is empty", register))),
            Some(v) => Ok(v),
        }
    }

    pub fn run(&mut self) -> Result<Value, VMError> {
        let mut frame = self.current.clone();
        let scope = frame.scope_id;
        let scope = match self.scopes.get(scope) {
            None => return Err(VMError::ScopeError(format!("Scope {} does not exist", scope))),
            Some(s) => s.clone(),
        };
        let len = scope.instructions.len();
        loop {
            if frame.pc >= len {
                // TODO this should probably be an error requiring explicit halt, halt 0 returns none
                break;
            }

            let instruction = frame.next_instruction(&scope);
            match instruction {
                Instruction::Halt(r) => {
                    return Ok(self.remove_register(&r)?)
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
                    if self.scopes.len() >= scope_index {
                        return Err(VMError::ScopeDoesNotExist(format!("{} does not exist", scope_index)))
                    }
                    let current = self.current.to_owned();
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
            }
        }
        Ok(Value::None)
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::VMBuilder;
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
}
