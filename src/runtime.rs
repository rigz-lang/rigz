use crate::ast::{parse, Parser, Program, ValidationError};
use crate::modules::VMModule;
use crate::token::ParsingError;
use rigz_vm::{Module, VMError, Value, VM};
use std::panic;

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

    /// Does not include default modules
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
        let vm = program.create_vm()?;
        Ok(Runtime { vm })
    }

    /// Does not include default modules
    pub fn create_unverified_with_modules(
        input: &'vm str,
        modules: Vec<impl Module<'vm> + 'static>,
    ) -> Result<Self, RuntimeError> {
        let mut parser = Parser::prepare(input).map_err(|e| e.into())?;
        let program = parser.parse().map_err(|e| e.into())?;
        let vm = program.create_vm_with_modules(modules)?;
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

pub fn eval_show_vm(input: &str) -> Result<(VM, Value), (Option<VM>, RuntimeError)> {
    let input = match runtime_parse(input) {
        Ok(p) => p,
        Err(e) => return Err((None, e.into())),
    };

    let mut vm: VM = match input.create_verified_vm() {
        Ok(v) => v,
        Err(e) => return Err((None, e.into())),
    };
    match vm.eval() {
        Ok(v) => Ok((vm, v)),
        Err(e) => Err((Some(vm), e.into())),
    }
}

pub fn eval_debug_vm(input: &str) -> Result<(VM, Value), (Option<VM>, RuntimeError)> {
    let input = match runtime_parse(input) {
        Ok(p) => p,
        Err(e) => return Err((None, e.into())),
    };

    let mut vm: VM = match input.create_verified_vm() {
        Ok(v) => v,
        Err(e) => return Err((None, e.into())),
    };
    // println!("VM Initial State - {vm:#?}");
    match vm.eval() {
        Ok(v) => Ok((vm, v)),
        Err(e) => Err((Some(vm), e.into())),
    }
}

pub fn repl_eval(input: &str) -> Result<Value, RuntimeError> {
    let input = runtime_parse(input)?;

    let mut vm: VM = input.create_vm()?;
    vm.eval().map_err(|e| e.into())
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use crate::runtime::{eval, eval_debug_vm, eval_show_vm};
    use rigz_vm::{Number, VMError, Value};

    macro_rules! run_expected {
        ($($name:ident($input:literal = $expected:expr))*) => {
            $(
                 #[test]
                fn $name() {
                    let input = $input;
                    let v = eval(input);
                    assert_eq!(v, Ok($expected), "VM eval failed {input}");
                }
            )*
        };
    }

    macro_rules! run_show_vm {
        ($($name:ident($input:literal = $expected:expr))*) => {
            $(
                 #[test]
                fn $name() {
                    let input = $input;
                    match eval_debug_vm(input) {
                        Ok((vm, v)) => {
                            assert_eq!(v, $expected, "VM eval failed {input}\n{vm:#?}")
                        }
                        Err((vm, err)) => match vm {
                            None => {
                                assert!(false, "Failed to parse input {err:?} - {input}")
                            }
                            Some(v) => {
                                assert!(false, "VM eval failed {err:?} - {input}\n{v:#?}")
                            }
                        }
                    };

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

        run_error! {
            /*
                function definitions claim registers for args & outputs,
                using get_register is very risky, VM.last is the best options
                VM.first will be altered if an imported module has a default implementation
            */
            vm_register_invalid("import VM; VM.get_register 1" = VMError::EmptyRegister("R1 is empty".to_string()).into())
            error("error 'something went wrong'" = VMError::RuntimeError("something went wrong".into()).into())
            stack_overflow(r#"fn foo
                foo
            end
            foo
            "# = VMError::RuntimeError("Stack overflow: exceeded 1024".into()).into())
        }
    }

    mod valid {
        use super::*;
        use indexmap::IndexMap;

        run_expected! {
            raw_value("'Hello World'" = Value::String("Hello World".to_string()))
            addition("2 + 2" = Value::Number(4.into()))
            complex_expression_ignore_precedence("1 + 2 * 3 - 4 / 5" = Value::Number(1.into()))
            ignore_precedence("2 + 1 * 3" = Value::Number(9.into()))
            paren_precedence("2 + (1 * 3)" = Value::Number(5.into()))
            assign("a = 3 * 2; a" = Value::Number(6.into()))
            assign_add("a = 1 + 2; a + 2" = Value::Number(5.into()))
            mutable_add("mut a = 4; a += 2; a" = Value::Number(6.into()))
            to_s("1.to_s" = Value::String("1".to_string()))
            unary_not("!1" = Value::Bool(false))
            unary_neg("-2.5" = Value::Number((-2.5).into()))
            vm_last_register("import VM; a = 1; VM.last" = Value::Number(1.into()))
            // VM.first will not be 27 if an imported module has a default implementation
            vm_first_register("import VM; a = 27; VM.first" = Value::Number(27.into()))
            binary_expr_function_call(r#"
            fn foo(number: Number) -> Number
                number * 2
            end
            a = 3
            1.to_s + foo a
            "# = 7.into())
            call_function_multiple_times(r#"
            fn foo(number: Number) -> Number
                number * 2
            end
            a = 1
            foo (foo (foo a))
            "# = 8.into())
            call_extension_function_mutable(r#"
            fn mut Number.foo -> mut Self
                self *= 3
                self
            end
            mut b = 2
            b.foo
            b
            "# = 6.into())
            call_extension_function_multiple_times_inline(r#"
            fn mut Number.foo -> mut Self
                self *= 3
                self
            end
            mut a = 2
            ((a.foo).foo).foo
            a
            "# = 54.into())
            call_extension_function_multiple_times(r#"
            fn mut Number.bah -> mut Self
                self *= 3
                self
            end
            mut f = 4.2
            f.bah
            f.bah
            f.bah
            f
            "# = 113.4.into())
            instance_get(r#"
                m = {a = {b = {c = 1}}}
                m.a.b.c
            "# = 1.into())
            binary_expr_instance_call(r#"
            11.4 + 1.2.ceil
            "# = 13.4.into())
            create_dynamic_list(r#"
                [{d = 1}]
            "# = Value::List(vec![Value::Map(IndexMap::from([("d".into(), 1.into())]))]))
            call_extension_function_multiple_times_inline_no_parens(r#"
            fn mut String.foo -> mut Self
                self += "h"
                self
            end
            mut a = ""
            a.foo.foo.foo
            a == "hhh"
            "# = true.into())
            call_module_extension_function_in_extension_scope(r#"
            fn String.foo -> Self
                "h" + self.to_s
            end
            "i".foo
            "# = "hi".into())
            lte("6 <= 1" = false.into())
            gte("6 >= 1" = true.into())
            if_true(r#"if 0 == none
                14
            end"# = 14.into())
            if_false(r#"if 1 == "abc"
                14
            end"# = Value::None.into())
            to_json("{a=5}.to_json" = r#"{"a":5}"#.into())
            json_parse("JSON.parse '5'" = 5.into())
            // memoization lifecycle
            // fib_recursive_dynamic_programming(r#"
            // @memo
            // fn fib(n: Number) -> Number
            //     if n <= 1
            //         n
            //     else
            //         a = (fib n - 1)
            //         b = (fib n - 2)
            //         a + b
            //     end
            // end
            // fib 6
            // fib 6
            // "# = 8.into())
            if_else_true(r#"if 0 == ""
                42
            else
                37
            end"# = 42.into())
            if_else_false(r#"if !true
                42
            else
                1 + 2
            end"# = 3.into())
            factorial(r#"
            fn factorial(n: Number)
                if n == 0
                    1
                else
                    n * factorial n - 1
                end
            end
            factorial 4
            "#=24.into())
            // todo variable should not be necessary for fib calls
            fib_recursive(r#"
            fn fib(n: Number) -> Number
                if n <= 1
                    n
                else
                    b = n - 2
                    (fib n - 1) + fib b
                end
            end
            fib 6
            "# = 8.into())
            self_fib_recursive(r#"
            fn Number.fib -> Number
                if self <= 1
                    self
                else
                    b = self - 2
                    (self - 1).fib + b.fib
                end
            end
            6.fib
            "# = 8.into())
        }
    }


    mod debug {
        use super::*;

        run_show_vm! {

        }
    }
}
