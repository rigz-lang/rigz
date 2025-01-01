use crate::Register;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Binary {
    pub op: BinaryOperation,
    pub lhs: Register,
    pub rhs: Register,
    pub output: Register,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BinaryAssign {
    pub op: BinaryOperation,
    pub lhs: Register,
    pub rhs: Register,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    Gte,
    Gt,
    Lt,
    Lte,
    Elvis,
}

impl Display for BinaryOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperation::Add => write!(f, "+"),
            BinaryOperation::Sub => write!(f, "-"),
            BinaryOperation::Mul => write!(f, "*"),
            BinaryOperation::Div => write!(f, "/"),
            BinaryOperation::Rem => write!(f, "%"),
            BinaryOperation::Shr => write!(f, ">>"),
            BinaryOperation::Shl => write!(f, "<<"),
            BinaryOperation::BitOr => write!(f, "|"),
            BinaryOperation::BitAnd => write!(f, "&"),
            BinaryOperation::BitXor => write!(f, "^"),
            BinaryOperation::Or => write!(f, "||"),
            BinaryOperation::And => write!(f, "&&"),
            BinaryOperation::Xor => write!(f, "^"),
            BinaryOperation::Eq => write!(f, "=="),
            BinaryOperation::Neq => write!(f, "!="),
            BinaryOperation::Gte => write!(f, ">="),
            BinaryOperation::Gt => write!(f, ">"),
            BinaryOperation::Lt => write!(f, "<"),
            BinaryOperation::Lte => write!(f, "<="),
            BinaryOperation::Elvis => write!(f, "?:"),
        }
    }
}
