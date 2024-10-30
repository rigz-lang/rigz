mod runtime {
    #[allow(unused_imports)] // used by macro
    use rigz_runtime::runtime::{eval, eval_print_vm};
    use rigz_ast::{VMError, Value};

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

    #[allow(unused_macros)]
    macro_rules! run_debug_vm {
        ($($name:ident($input:literal = $expected:expr))*) => {
            $(
                 #[test]
                fn $name() {
                    let input = $input;
                    let v = eval_print_vm(input);
                    assert_eq!(v, Ok($expected), "VM eval failed {input}");
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
                    assert_eq!(v, Err($expected.into()), "Failed to parse input {}", input)
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
            // last statement must be an expression
            assign("a = 3 * 2")
            var_once_in_fn_def("fn foo(var foo, var bar) = none")
        }

        run_error! {
            // todo better error message here, ideally this fails during validation
            import_required("1.to_json" = VMError::UnsupportedOperation("Cannot read to_json for 1".to_string()))
            /*
                function definitions claim registers for args & outputs,
                using get_register is very risky, VM.last is the best options
                VM.first will be altered if an imported module has a default implementation
            */
            vm_register_invalid("import VM; VM.get_register 42" = VMError::EmptyRegister("R42 is empty".to_string()))
            raise_error("raise 'something went wrong'" = VMError::RuntimeError("something went wrong".into()))
            assert("assert_eq 1, 2" = VMError::RuntimeError("Assertion Failed\n\t\tLeft: 1\n\t\tRight: 2".to_string()))
            stack_overflow(r#"fn foo
                foo
            end
            foo
            "# = VMError::RuntimeError("Stack overflow: exceeded 1024".into()))
        }
    }

    mod valid {
        use rigz_ast::IndexMap;
        use super::*;

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
            create_list(r#"
            [1, 2, 3, 4]
            "# = vec![1.into(), 2.into(), 3.into(), 4.into()].into())
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
            to_json("import JSON; {a=5}.to_json" = r#"{"a":5}"#.into())
            json_parse("import JSON; JSON.parse '5'" = 5.into())
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
            var_args_module(r#"
            let a = []
            a.with 1, 2, 3
            "# = Value::List(vec![1.into(), 2.into(), 3.into()]))
            var_args_module_str(r#"
            let a = ""
            a.with 1, 2, 3
            "# = "123".into())
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
                    b = (self - 2)
                    (self - 1).fib + b.fib
                end
            end
            6.fib
            "# = 8.into())
        }
    }
}
