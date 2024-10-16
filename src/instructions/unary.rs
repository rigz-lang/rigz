use crate::instructions::Clear;
use crate::{Register, Reverse, VMError, Value, VM};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unary {
    pub op: UnaryOperation,
    pub from: Register,
    pub output: Register,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOperation {
    Neg,
    Not,
    Reverse,
    Print,
    EPrint,
    PrintLn,
    EPrintLn,
}

impl<'vm> VM<'vm> {
    pub fn apply_unary(
        &mut self,
        unary_operation: UnaryOperation,
        val: Value,
        output: Register,
    ) -> Result<(), VMError> {
        match unary_operation {
            UnaryOperation::Neg => {
                self.insert_register(output, -val);
            }
            UnaryOperation::Not => {
                self.insert_register(output, !val);
            }
            UnaryOperation::PrintLn => {
                println!("{}", val);
                self.insert_register(output, val);
            }
            UnaryOperation::EPrintLn => {
                eprintln!("{}", val);
                self.insert_register(output, val);
            }
            UnaryOperation::Print => {
                print!("{}", val);
                self.insert_register(output, val);
            }
            UnaryOperation::EPrint => {
                eprint!("{}", val);
                self.insert_register(output, val);
            }
            UnaryOperation::Reverse => {
                self.insert_register(output, val.reverse());
            }
        }
        Ok(())
    }

    pub fn handle_unary(&mut self, unary: Unary) -> Result<(), VMError> {
        let Unary { op, from, output } = unary;
        let val = self.resolve_register(from)?;
        self.apply_unary(op, val, output)
    }

    pub fn handle_unary_assign(&mut self, unary: Unary) -> Result<(), VMError> {
        let Unary { op, from, .. } = unary;
        let val = self.resolve_register(from)?;
        self.apply_unary(op, val, from)
    }

    pub fn handle_unary_clear(&mut self, unary: Unary, clear: Clear) -> Result<(), VMError> {
        let Unary { op, from, output } = unary;
        let val = match clear {
            Clear::One(c) if c != from => {
                return Err(VMError::RuntimeError(format!(
                    "Invalid Register Passed to unary_clear: {} != {}",
                    c, from
                )))
            }
            Clear::One(c) => self.remove_register_eval_scope(c)?,
            c => {
                return Err(VMError::RuntimeError(format!(
                    "Invalid Option Passed to unary_clear: {:?}",
                    c
                )))
            }
        };
        self.apply_unary(op, val, output)
    }
}
