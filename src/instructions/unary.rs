use crate::instructions::Clear;
use crate::{Register, Reverse, VMError, Value, VM};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unary {
    pub op: UnaryOperation,
    pub from: Register,
    pub output: Register,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

fn eval_unary(unary_operation: UnaryOperation, val: Value) -> Value {
    match unary_operation {
        UnaryOperation::Neg => -val,
        UnaryOperation::Not => !val,
        UnaryOperation::PrintLn => {
            println!("{}", val);
            Value::None
        }
        UnaryOperation::EPrintLn => {
            eprintln!("{}", val);
            Value::None
        }
        UnaryOperation::Print => {
            print!("{}", val);
            Value::None
        }
        UnaryOperation::EPrint => {
            eprint!("{}", val);
            Value::None
        }
        UnaryOperation::Reverse => val.reverse(),
    }
}

impl<'vm> VM<'vm> {
    pub fn apply_unary(&mut self, unary_operation: UnaryOperation, val: Value, output: Register) {
        let val = eval_unary(unary_operation, val);
        self.insert_register(output, val.into())
    }

    pub fn handle_unary(&mut self, unary: Unary) {
        let Unary { op, from, output } = unary;
        let val = self.resolve_register(from);
        self.apply_unary(op, val, output);
    }

    pub fn handle_unary_assign(&mut self, unary: UnaryAssign) {
        let UnaryAssign { op, from } = unary;
        match self.update_register(from, |v| {
            *v = eval_unary(op, v.clone());
            Ok(None)
        }) {
            Ok(_) => {}
            Err(e) => self.insert_register(from, e.into()),
        }
    }

    pub fn handle_unary_clear(&mut self, unary: Unary, clear: Clear) {
        let Unary { op, from, output } = unary;
        let val = match clear {
            Clear::One(c) if c != from => VMError::RuntimeError(format!(
                "Invalid Register Passed to unary_clear: {} != {}",
                c, from
            ))
            .into(),
            Clear::One(c) => self.remove_register_eval_scope(c),
            c => VMError::RuntimeError(format!("Invalid Option Passed to unary_clear: {:?}", c))
                .into(),
        };
        self.apply_unary(op, val, output);
    }
}
