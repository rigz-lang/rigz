use crate::modules::std_lib::StdLibModule;
use crate::modules::vm::VMModule;
use crate::{Parser};
use rigz_vm::{Scope, VMBuilder, VMError, Value, VM};

pub struct Runtime<'run> {
    vm: VM<'run>,
}

impl<'run> Runtime<'run> {
    pub fn prepare(input: &str) -> Result<Runtime, VMError> {
        let mut builder = VMBuilder::new();
        builder.register_module(VMModule {});
        builder.register_module(StdLibModule {});
        let vm = match Parser::parse_with_builder(input, builder) {
            Ok(vm) => vm,
            Err(e) => {
                return Err(VMError::ParseError(
                    format!("Failed to parse input: {:?}", e),
                    0,
                    usize::MAX,
                ))
            }
        };

        Ok(Runtime { vm })
    }

    pub fn run(&mut self) -> Result<Value, VMError> {
        self.vm.run()
    }

    pub fn register_value(&mut self, index: usize) -> Option<Value> {
        match self.vm.registers.get(&index) {
            None => None,
            Some(s) => Some(s.clone()),
        }
    }

    pub fn scope(&mut self, index: usize) -> Option<Scope> {
        match self.vm.scopes.get(index) {
            None => None,
            Some(s) => Some(s.clone()),
        }
    }

    pub fn run_repl(&mut self, input: &'run str) -> Result<Value<'run>, VMError> {
        todo!()
        // let mut vm = match Parser::parse(input) {
        //     Ok(vm) => vm,
        //     Err(e) => {
        //         return Err(VMError::ParseError(
        //             format!("Failed to parse input: {:?}", e),
        //             0,
        //             usize::MAX,
        //         ))
        //     }
        // };
        // let v = vm.run()?;
        // Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rigz_vm::{Number, Value};

    macro_rules! test_run {
        ($($name:ident $input:literal = $expected:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let input = $input;
                    let mut r = Runtime::prepare(input).unwrap();
                    let v = r.run().unwrap();
                    assert_eq!(v, $expected)
                }
            )*
        };
    }

    test_run! {
        basic "1 + 2" = Value::Number(3.into()),
        complex "1 + 2 * 3" = Value::Number(7.into()),
        complex_ignore_precedence "3 * 2 + 1" = Value::Number(9.into()),
        assign "a = 1 + 2" = Value::Number(3.into()),
        assign_add "a = 1 + 2; a + 2" = Value::Number(5.into()),
        unary_not "!1" = Value::Number(Number::Int(!1)),
        vm_register "__VM.get_register 0" = Value::None,
        define_function r#"
            fn hello
              "hi there"
            end
            hello"# = Value::String("hi there".into()),
        define_function_args r#"
            fn add(a, b, c)
              a + b + c
            end
            add 1, 2, 3"# = Value::Number(Number::Int(6)),
    }
}
