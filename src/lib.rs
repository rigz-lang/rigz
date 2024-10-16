extern crate core;

mod builder;
mod value;
mod number;

use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::ops::Index;
use indexmap::IndexMap;
use crate::value::Value;

pub use builder::VMBuilder;

pub trait Rev {
    type Output;

    fn rev(self) -> Self::Output;
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
}

#[derive(Clone, Debug)]
pub enum UnaryOperation {
    Neg,
    Not,
    Rev,
    Print
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
    bit_registers: IndexMap<usize, bool>,
}

impl VM {
    pub fn remove_register(&mut self, register: &Register) -> Result<Value, VMError> {
        match self.registers.shift_remove(register) {
            None => Err(VMError::EmptyRegister(format!("R{} is empty", register))),
            Some(v) => Ok(v),
        }
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        let mut frame = self.current.clone();
        let scope = frame.scope_id;
        let scope = match self.scopes.get(scope) {
            None => return Err(VMError::ScopeError(format!("Scope {} does not exist", scope))),
            Some(s) => s.clone(),
        };
        let len = scope.instructions.len();
        loop {
            if frame.pc >= len {
                break;
            }

            let instruction = frame.next_instruction(&scope);
            match instruction {
                Instruction::Unary { op, from, output } => {
                    let val = match self.registers.shift_remove(&from) {
                        None => return Err(VMError::EmptyRegister(format!("R{} is empty", from))),
                        Some(v) => v,
                    };
                    match op {
                        UnaryOperation::Neg => {
                            self.registers.insert(output, -val);
                        }
                        UnaryOperation::Not => {
                            self.registers.insert(output, !val);
                        },
                        UnaryOperation::Print => {
                            println!("{}", val);
                            self.registers.insert(output, val);
                        }
                        UnaryOperation::Rev => {
                            self.registers.insert(output, val.rev());
                        }
                    }
                }
                Instruction::Binary { op, lhs, rhs, output } => {
                    let lhs = self.remove_register(&lhs)?;
                    let rhs = self.remove_register(&rhs)?;
                    let v = match op {
                        BinaryOperation::Add => lhs + rhs,
                        _ => todo!()
                    };
                    self.registers.insert(output, v);
                }
                Instruction::Load(r, v) => {
                    self.registers.insert(r, v);
                }
                Instruction::Copy(from, to) => {
                    let copy = match self.registers.get(&from) {
                        None => return Err(VMError::EmptyRegister(format!("R{} is empty", from))),
                        Some(s) => s.clone()
                    };
                    self.registers.insert(to, copy);
                }
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::VMBuilder;
    use crate::number::Number;
    use crate::value::Value;
    use super::*;

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
}
