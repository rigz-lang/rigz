use crate::ast::{parse, Parser, ValidationError};
use crate::token::LexingError;
use rigz_vm::{VMError, Value, VM};

pub struct Runtime<'vm> {
    vm: VM<'vm>,
}

#[derive(Debug)]
pub enum RuntimeError {
    Lex(LexingError),
    Validation(ValidationError),
    Run(VMError),
}

impl Into<RuntimeError> for LexingError {
    fn into(self) -> RuntimeError {
        RuntimeError::Lex(self)
    }
}

impl Into<RuntimeError> for VMError {
    fn into(self) -> RuntimeError {
        RuntimeError::Run(self)
    }
}

impl Into<RuntimeError> for ValidationError {
    fn into(self) -> RuntimeError {
        RuntimeError::Validation(self)
    }
}

impl<'vm> Runtime<'vm> {
    pub fn create(input: &'vm str) -> Result<Self, RuntimeError> {
        let mut parser = Parser::prepare(input).map_err(|e| e.into())?;
        let program = parser.parse().map_err(|e| e.into())?;
        let intermediate: Result<VM<'vm>, RuntimeError> = program.try_into();
        Ok(Runtime { vm: intermediate? })
    }

    pub fn run(&mut self) -> Result<Value, RuntimeError> {
        self.vm.eval().map_err(|e| e.into())
    }
}

pub fn eval(input: &str) -> Result<Value, RuntimeError> {
    let input = match parse(input) {
        Ok(p) => p,
        Err(e) => {
            return Err(VMError::ParseError(
                format!("Failed to parse input: {:?}", e),
                0,
                usize::MAX,
            )
            .into())
        }
    };

    let mut vm: VM = input.try_into()?;
    vm.run().map_err(|e| e.into())
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use crate::runtime::eval;
    use rigz_vm::{Number, Value};

    macro_rules! run_expected {
        ($($name:ident($input:literal = $expected:expr))*) => {
            $(
                 #[test]
                fn $name() {
                    let input = $input;
                    let v = eval(input).expect("Failed to parse input");
                    assert_eq!(v, $expected)
                }
            )*
        };
    }

    macro_rules! run_invalid {
        ($($name:ident($input:literal))*) => {
            $(
                 #[test]
                fn $name() {
                    let input = $input;
                    let v = eval(input).expect_err("Successfully parsed invalid input");
                    assert!(true)
                }
            )*
        };
    }

    mod invalid {
        use super::*;

        run_invalid! {
            assign("a = 3 * 2")
        }
    }

    mod valid {
        use super::*;

        run_expected! {
            raw_value("'Hello World'" = Value::String("Hello World".to_string()))
            addition("2 + 2" = Value::Number(Number(4.0)))
            ignore_precedence("2 + 1 * 3" = Value::Number(Number(6.0)))
            paren_precedence("(2 + 1) * 3)" = Value::Number(Number(5.0)))
            assign("a = 3 * 2; a" = Value::Number(Number(6.0)))
        }
    }
}
