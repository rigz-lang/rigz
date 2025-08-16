use std::fmt::{Display, Formatter};

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BinaryAssignOperation {
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
    Xor
}

impl Display for BinaryAssignOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryAssignOperation::Add => write!(f, "+="),
            BinaryAssignOperation::Sub => write!(f, "-="),
            BinaryAssignOperation::Mul => write!(f, "*="),
            BinaryAssignOperation::Div => write!(f, "/="),
            BinaryAssignOperation::Rem => write!(f, "%="),
            BinaryAssignOperation::Shr => write!(f, ">>="),
            BinaryAssignOperation::Shl => write!(f, "<<="),
            BinaryAssignOperation::BitOr => write!(f, "|="),
            BinaryAssignOperation::BitAnd => write!(f, "&="),
            BinaryAssignOperation::BitXor => write!(f, "^="),
            BinaryAssignOperation::Or => write!(f, "||="),
            BinaryAssignOperation::And => write!(f, "&&="),
            BinaryAssignOperation::Xor => write!(f, "^="),
        }
    }
}

impl BinaryOperation {
    pub fn infix_priority(&self) -> (u8, u8) {
        match self {
            BinaryOperation::Eq
            | BinaryOperation::Neq
            | BinaryOperation::Gte
            | BinaryOperation::Gt
            | BinaryOperation::Lt
            | BinaryOperation::Lte
            | BinaryOperation::Elvis => (10, 9),
            BinaryOperation::Rem => (12, 11),
            BinaryOperation::Or | BinaryOperation::Shr | BinaryOperation::Shl => (6, 5),
            BinaryOperation::And | BinaryOperation::Xor => (7, 8),
            _ => (1, 2),
        }
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
