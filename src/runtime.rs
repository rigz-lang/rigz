use crate::ast::{parse, Parser, Program, ValidationError};
use crate::token::ParsingError;
use rigz_vm::{Module, VMError, Value, VM};

pub struct Runtime<'vm> {
    vm: VM<'vm>,
}

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    Parse(ParsingError),
    Validation(ValidationError),
    Run(VMError),
}

impl Into<RuntimeError> for ParsingError {
    #[inline]
    fn into(self) -> RuntimeError {
        RuntimeError::Parse(self)
    }
}

impl Into<RuntimeError> for VMError {
    #[inline]
    fn into(self) -> RuntimeError {
        RuntimeError::Run(self)
    }
}

impl Into<RuntimeError> for ValidationError {
    #[inline]
    fn into(self) -> RuntimeError {
        RuntimeError::Validation(self)
    }
}

impl<'vm> Runtime<'vm> {
    pub fn create(input: &'vm str) -> Result<Self, RuntimeError> {
        let mut parser = Parser::prepare(input).map_err(|e| e.into())?;
        let program = parser.parse().map_err(|e| e.into())?;
        let intermediate = program.create_verified_vm();
        Ok(Runtime { vm: intermediate? })
    }

    pub fn create_with_modules(
        input: &'vm str,
        modules: Vec<impl Module<'vm> + 'static>,
    ) -> Result<Self, RuntimeError> {
        let mut parser = Parser::prepare(input).map_err(|e| e.into())?;
        let program = parser.parse().map_err(|e| e.into())?;
        let intermediate = program.create_verified_vm_with_modules(modules);
        Ok(Runtime { vm: intermediate? })
    }

    pub fn create_unverified(input: &'vm str) -> Result<Self, RuntimeError> {
        let mut parser = Parser::prepare(input).map_err(|e| e.into())?;
        let program = parser.parse().map_err(|e| e.into())?;
        let vm = program.create_vm();
        Ok(Runtime { vm })
    }

    pub fn create_unverified_with_modules(
        input: &'vm str,
        modules: Vec<impl Module<'vm> + 'static>,
    ) -> Result<Self, RuntimeError> {
        let mut parser = Parser::prepare(input).map_err(|e| e.into())?;
        let program = parser.parse().map_err(|e| e.into())?;
        let vm = program.create_vm_with_modules(modules);
        Ok(Runtime { vm })
    }

    pub fn run(&mut self) -> Result<Value, RuntimeError> {
        self.vm.eval().map_err(|e| e.into())
    }
}

fn runtime_parse(input: &str) -> Result<Program<'_>, RuntimeError> {
    match parse(input) {
        Ok(p) => Ok(p),
        Err(e) => Err(e.into()),
    }
}

pub fn eval(input: &str) -> Result<Value, RuntimeError> {
    let input = runtime_parse(input)?;

    let mut vm: VM = input.create_verified_vm()?;
    vm.eval().map_err(|e| e.into())
}

pub fn repl_eval(input: &str) -> Result<Value, RuntimeError> {
    let input = runtime_parse(input)?;

    let mut vm: VM = input.create_vm();
    vm.eval().map_err(|e| e.into())
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
                    let v = eval(input);
                    assert_eq!(v, Ok($expected), "Failed to parse input {}", input)
                }
            )*
        };
    }

    macro_rules! run_error {
        ($($name:ident($input:literal = $expected:expr))*) => {
            $(
                 #[test]
                fn $name() {
                    let input = $input;
                    let v = eval(input);
                    assert_eq!(v, Err($expected), "Failed to parse input {}", input)
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
                    let v = eval(input);
                    assert!(v.is_err(), "Successfully parsed invalid input: {}", input)
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
        use rigz_vm::VMError;
        use super::*;

        run_expected! {
            raw_value("'Hello World'" = Value::String("Hello World".to_string()))
            addition("2 + 2" = Value::Number(4.into()))
            complex_expression_ignore_precedence("1 + 2 * 3 - 4 / 5" = Value::Number(1.into()))
            ignore_precedence("2 + 1 * 3" = Value::Number(9.into()))
            paren_precedence("2 + (1 * 3)" = Value::Number(5.into()))
            assign("a = 3 * 2; a" = Value::Number(6.into()))
            assign_add("a = 1 + 2; a + 2" = Value::Number(5.into()))
            to_s("1.to_s" = Value::String("1".to_string()))
            unary_not("!1" = Value::Bool(false))
            unary_neg("-2.5" = Value::Number((-2.5).into()))
            // arg is loaded into 0th register, using Number to be explicit however 0 is also equal to none
            vm_register("VM.get_register 0" = Value::Number(0.into()))
        }

        run_error! {
            vm_register_invalid("VM.get_register 1" = VMError::EmptyRegister("R1 is empty".to_string()).into())
        }
    }
}
