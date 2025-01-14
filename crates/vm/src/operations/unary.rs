use crate::{Snapshot, VMError};
use std::fmt::{Display, Formatter};
use std::vec::IntoIter;

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

impl Snapshot for UnaryOperation {
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
            0 => UnaryOperation::Neg,
            1 => UnaryOperation::Not,
            2 => UnaryOperation::Reverse,
            3 => UnaryOperation::Print,
            4 => UnaryOperation::EPrint,
            5 => UnaryOperation::PrintLn,
            6 => UnaryOperation::EPrintLn,
            b => {
                return Err(VMError::RuntimeError(format!(
                    "Illegal UnaryOperation byte {b} - {location}"
                )))
            }
        };
        Ok(op)
    }
}
