use crate::{BinaryAssignOperation, BinaryOperation, Snapshot, UnaryOperation, VMError};
use std::fmt::Display;
use std::vec::IntoIter;

impl Snapshot for BinaryOperation {
    fn as_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            None => {
                return Err(VMError::runtime(format!(
                    "Missing BinaryOperation byte {location}"
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
                return Err(VMError::runtime(format!(
                    "Illegal BinaryOperation byte {b} - {location}"
                )))
            }
        };
        Ok(op)
    }
}

impl Snapshot for BinaryAssignOperation {
    fn as_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            None => {
                return Err(VMError::runtime(format!(
                    "Missing BinaryAssignOperation byte {location}"
                )))
            }
            Some(b) => b,
        };

        let op = match next {
            0 => BinaryAssignOperation::Add,
            1 => BinaryAssignOperation::Sub,
            2 => BinaryAssignOperation::Mul,
            3 => BinaryAssignOperation::Div,
            4 => BinaryAssignOperation::Rem,
            5 => BinaryAssignOperation::Shr,
            6 => BinaryAssignOperation::Shl,
            7 => BinaryAssignOperation::BitOr,
            8 => BinaryAssignOperation::BitAnd,
            9 => BinaryAssignOperation::BitXor,
            10 => BinaryAssignOperation::Or,
            11 => BinaryAssignOperation::And,
            12 => BinaryAssignOperation::Xor,
            b => {
                return Err(VMError::runtime(format!(
                    "Illegal BinaryAssignOperation byte {b} - {location}"
                )))
            }
        };
        Ok(op)
    }
}

impl Snapshot for UnaryOperation {
    fn as_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        let next = match bytes.next() {
            None => {
                return Err(VMError::runtime(format!(
                    "Missing UnaryOperation byte {location}"
                )))
            }
            Some(b) => b,
        };

        let op = match next {
            0 => UnaryOperation::Neg,
            1 => UnaryOperation::Not,
            2 => UnaryOperation::Reverse,
            b => {
                return Err(VMError::runtime(format!(
                    "Illegal UnaryOperation byte {b} - {location}"
                )))
            }
        };
        Ok(op)
    }
}
