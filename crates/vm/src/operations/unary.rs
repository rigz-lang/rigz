use crate::Register;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Unary {
    pub op: UnaryOperation,
    pub from: Register,
    pub output: Register,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct UnaryAssign {
    pub op: UnaryOperation,
    pub from: Register,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnaryOperation {
    Neg,
    Not,
    Reverse,
    Print,
    EPrint,
    PrintLn,
    EPrintLn,
}

impl Display for UnaryOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOperation::Neg => write!(f, "-"),
            UnaryOperation::Not => write!(f, "!"),
            UnaryOperation::Reverse => write!(f, "rev"),
            UnaryOperation::Print => write!(f, "print"),
            UnaryOperation::EPrint => write!(f, "eprint"),
            UnaryOperation::PrintLn => write!(f, "println"),
            UnaryOperation::EPrintLn => write!(f, "eprintln"),
        }
    }
}
