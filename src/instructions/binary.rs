use crate::instructions::Clear;
use crate::{Logical, Register, VMError, Value, VM};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binary {
    pub op: BinaryOperation,
    pub lhs: Register,
    pub rhs: Register,
    pub output: Register,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
}

impl<'vm> VM<'vm> {
    pub fn apply_binary(
        &mut self,
        binary_operation: BinaryOperation,
        lhs: Value<'vm>,
        rhs: Value<'vm>,
        output: Register,
    ) -> Result<(), VMError> {
        let v = match binary_operation {
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
            BinaryOperation::Xor => lhs.xor(rhs),
            BinaryOperation::Gt => Value::Bool(lhs > rhs),
            BinaryOperation::Gte => Value::Bool(lhs >= rhs),
            BinaryOperation::Lt => Value::Bool(lhs < rhs),
            BinaryOperation::Lte => Value::Bool(lhs <= rhs),
        };

        self.insert_register(output, v);
        Ok(())
    }

    pub fn handle_binary(&mut self, binary: Binary) -> Result<(), VMError> {
        let Binary {
            op,
            lhs,
            rhs,
            output,
        } = binary;
        let lhs = self.get_register(lhs)?;
        let rhs = self.get_register(rhs)?;
        self.apply_binary(op, lhs, rhs, output)
    }

    pub fn handle_binary_assign(&mut self, binary: Binary) -> Result<(), VMError> {
        let Binary { op, lhs, rhs, .. } = binary;
        let v = self.get_register(lhs)?;
        let rhs = self.get_register(rhs)?;
        self.apply_binary(op, v, rhs, lhs)
    }

    pub fn handle_binary_clear(&mut self, binary: Binary, clear: Clear) -> Result<(), VMError> {
        let Binary {
            op,
            lhs,
            rhs,
            output,
        } = binary;
        let (lhs, rhs) = match clear {
            Clear::One(c) if c == rhs => {
                (self.get_register(lhs), self.remove_register_eval_scope(c))
            }
            Clear::One(c) if c == lhs => {
                (self.remove_register_eval_scope(lhs), self.get_register(c))
            }
            Clear::Two(c1, c2) if c1 == lhs && c2 == rhs => (
                self.remove_register_eval_scope(c1),
                self.remove_register_eval_scope(c2),
            ),
            Clear::Two(c1, c2) if c2 == lhs && c1 == rhs => (
                self.remove_register_eval_scope(c2),
                self.remove_register_eval_scope(c1),
            ),
            Clear::One(c) => {
                return Err(VMError::RuntimeError(format!(
                    "Invalid Register Passed to binary_clear: {} must be {} or {}",
                    c, lhs, rhs
                )))
            }
            Clear::Two(c1, c2) => {
                return Err(VMError::RuntimeError(format!(
                    "Invalid Registers Passed to binary_clear: {} and {} must be either {} or {}",
                    c1, c2, lhs, rhs
                )))
            }
            c => {
                return Err(VMError::RuntimeError(format!(
                    "Invalid Option Passed to binary_clear: {:?}",
                    c
                )))
            }
        };
        self.apply_binary(op, lhs?, rhs?, output)
    }
}