use rigz_vm::{VMError, Value, VM};
use crate::parse;

pub struct Runtime<'run> {
    vm: VM<'run>,
}

impl <'run> Runtime<'run> {
    pub fn run(input: &'run str) -> Result<Value<'run>, VMError> {
        let mut vm = match parse(input) {
            Ok(vm) => vm,
            Err(e) => return Err(VMError::ParseError(format!("Failed to parse input: {:?}", e), 0, usize::MAX))
        };
        vm.run()
    }

    pub fn run_repl(&mut self, input: &'run str) -> Result<Value<'run>, VMError> {
        let mut vm = match parse(input) {
            Ok(vm) => vm,
            Err(e) => return Err(VMError::ParseError(format!("Failed to parse input: {:?}", e), 0, usize::MAX))
        };
        vm.run()
    }
}

macro_rules! test_run {
    ($($name:ident $input:literal = $expected:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let input = $input;
                let v = Runtime::run(input).unwrap();
                assert_eq!(v, $expected)
            }
        )*
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use rigz_vm::{BinaryOperation, Instruction, Number, Scope, UnaryOperation, Value};

    test_run! {
        basic "1 + 2" = Value::Number(Number::Int(3)),
        complex "1 + 2 * 3" = Value::Number(Number::Int(9)),
        assign "a = 1 + 2" = Value::Number(Number::Int(3)),
        assign_add "a = 1 + 2; a + 2" = Value::Number(Number::Int(5)),
        unary_not "!1" = Value::Number(Number::Int(!1)),
    }
}