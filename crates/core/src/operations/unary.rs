use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnaryOperation {
    Neg,
    Not,
    Reverse
}

impl Display for UnaryOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOperation::Neg => write!(f, "-"),
            UnaryOperation::Not => write!(f, "!"),
            UnaryOperation::Reverse => write!(f, "rev"),
        }
    }
}
