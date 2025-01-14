use crate::{Snapshot, VMError};
use std::fmt::{Display, Formatter};
use std::vec::IntoIter;

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

impl Snapshot for BinaryOperation {
    fn as_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            None => {
                return Err(VMError::RuntimeError(format!(
                    "Missing UnaryOperation byte {location}"
                )))
            }
            Some(b) => b,
        };

        let op = match next {
            0 => BinaryOperation::Add,
            1 => BinaryOperation::Sub,
            2 => BinaryOperation::Mul,
            3 => BinaryOperation::Div,
            4 => BinaryOperation::Rem,
            5 => BinaryOperation::Shr,
            6 => BinaryOperation::Shl,
            7 => BinaryOperation::BitOr,
            8 => BinaryOperation::BitAnd,
            9 => BinaryOperation::BitXor,
            10 => BinaryOperation::Or,
            11 => BinaryOperation::And,
            12 => BinaryOperation::Xor,
            13 => BinaryOperation::Eq,
            14 => BinaryOperation::Neq,
            15 => BinaryOperation::Gte,
            16 => BinaryOperation::Gt,
            17 => BinaryOperation::Lt,
            18 => BinaryOperation::Lte,
            19 => BinaryOperation::Elvis,
            b => {
                return Err(VMError::RuntimeError(format!(
                    "Illegal UnaryOperation byte {b} - {location}"
                )))
            }
        };
        Ok(op)
    }
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
