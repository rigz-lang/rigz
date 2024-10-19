use crate::instructions::Clear;
use crate::{Logical, Register, VMError, Value, VM};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binary {
    pub op: BinaryOperation,
    pub lhs: Register,
    pub rhs: Register,
    pub output: Register,
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

impl<'vm> VM<'vm> {
    #[inline]
    pub fn apply_binary(
        &mut self,
        binary_operation: BinaryOperation,
        lhs: Value,
        rhs: Value,
        output: Register,
    ) {
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
            BinaryOperation::Elvis => lhs.elvis(rhs),
        };

        self.insert_register(output, v.into());
    }

    #[inline]
    pub fn handle_binary(&mut self, binary: Binary) {
        let Binary {
            op,
            lhs,
            rhs,
            output,
        } = binary;
        let lhs = self.resolve_register(lhs);
        let rhs = self.resolve_register(rhs);
        self.apply_binary(op, lhs, rhs, output);
    }

    pub fn handle_binary_assign(&mut self, binary: Binary) {
        let Binary { op, lhs, rhs, .. } = binary;
        let v = self.resolve_register(lhs);
        let rhs = self.resolve_register(rhs);
        self.apply_binary(op, v, rhs, lhs); // TODO measure cost of storing in same register vs impl *Assign trait
    }

    pub fn handle_binary_clear(&mut self, binary: Binary, clear: Clear) {
        let Binary {
            op,
            lhs,
            rhs,
            output,
        } = binary;
        let (lhs, rhs) = match clear {
            Clear::One(c) if c == rhs => (
                self.resolve_register(lhs),
                self.remove_register_eval_scope(c),
            ),
            Clear::One(c) if c == lhs => (
                self.remove_register_eval_scope(c),
                self.resolve_register(rhs),
            ),
            Clear::Two(c1, c2) if c1 == lhs && c2 == rhs => (
                self.remove_register_eval_scope(c1),
                self.remove_register_eval_scope(c2),
            ),
            Clear::Two(c1, c2) if c2 == lhs && c1 == rhs => (
                self.remove_register_eval_scope(c2),
                self.remove_register_eval_scope(c1),
            ),
            Clear::One(c) => (
                self.remove_register_eval_scope(c),
                VMError::RuntimeError(format!(
                    "Invalid Register Passed to binary_clear: {} must be {} or {}",
                    c, lhs, rhs
                ))
                .into(),
            ),
            Clear::Two(c1, c2) => {
                let v = VMError::RuntimeError(format!(
                    "Invalid Registers Passed to binary_clear: {} and {} must be either {} or {}",
                    c1, c2, lhs, rhs
                ))
                .to_value();
                (v.clone(), v)
            }
            c => {
                let v = VMError::RuntimeError(format!(
                    "Invalid Option Passed to binary_clear: {:?}",
                    c
                ))
                .to_value();
                (v.clone(), v)
            }
        };
        self.apply_binary(op, lhs, rhs, output);
    }
}
