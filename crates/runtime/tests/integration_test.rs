mod runtime {
    use rigz_ast::{VMError, Value};
    #[allow(unused_imports)] // used by macro
    use rigz_runtime::runtime::{eval, eval_print_vm};

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
                    pretty_env_logger::try_init();
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
        use super::*;
        use rigz_ast::IndexMap;

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
            named_for_positional(r#"
            fn foo(bar: Number) -> Number
                bar * 24
            end
            foo bar: 3
            "# = 72.into())
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
            "# = Value::List(vec![1.into(), 2.into(), 3.into(), 4.into()]))
            create_map(r#"
            {1, 2, 3, 4}
            "# = IndexMap::from([(1, 1), (2, 2), (3, 3), (4, 4)]).into())
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
            trailing_if(r#"v = 42; v if v.is_num"# = 42.into())
            instance_trailing_if_in_func(r#"
                fn foo(a)
                    a.to_i if a.is_num
                end
                foo 'a'
            "# = Value::None)
            if_false(r#"if 1 == "abc"
                14
            end"# = Value::None)
            to_json("import JSON; {a=5}.to_json" = r#"{"a":5}"#.into())
            json_parse("import JSON; JSON.parse '5'" = 5.into())
            format("format '{}', 1 + 2" = "3".into())
            format_parens("format('{}', 1 + 2)" = "3".into())
            is("1.is Number" = true.into())
            // fib_recursive_dynamic_programming(r#"
            // @memo
            // fn fib(n: Number) -> Number
            //     if n <= 1
            //         n
            //     else
            //         b = n - 2
            //         (fib n - 1) + fib b
            //     end
            // end
            // fib 5
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
            memo_factorial(r#"
            @memo
            fn factorial(n: Number)
                if n == 0
                    1
                else
                    n * factorial n - 1
                end
            end
            factorial 15
            "#=1307674368000_i64.into())
            var_args_module(r#"
            let a = []
            a.with 1, 2, 3
            "# = Value::List(vec![1.into(), 2.into(), 3.into()]))
            var_args_module_str(r#"
            let a = ""
            a.with 1, 2, 3
            "# = "123".into())
            lambda(r#"
            a = || 42
            a
            "# = 42.into())
            for_list(r#"[for v in [1, 2, 3]: v * v]"# = vec![1, 4, 9].into())
            for_list_exclude_nones(r#"[for v in [1, 2, 3, 'a', 'b']: v if v.is_num]"# = vec![1, 2, 3].into())
            for_map(r#"{for k, v in {1, 2, 3}: k, v if k % 2 == 0}"# = IndexMap::from([(2, 2)]).into())
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
            lambda_in_for_list_if_expression(r#"
            func = |v| v if v.is_num
            [for a in ['a', 'b', 'c', 1, 2, 3]: func a]
            "# = vec![1, 2, 3].into())
            lambda_in_for_list(r#"
            func = |v| v.is_num
            [for a in ['a', 'b', 'c', 1, 2, 3]: a if func a]
            "# = vec![1, 2, 3].into())
            trailing_if_false(r#"v = 'a'; v if v.is_num"# = Value::None)
            instance_trailing_if(r#"a = 'a'; a.to_i if a.is_num"# = Value::None)
            // todo parens shouldn't be required to pass in lambdas
            filter(r#"[1, 2, 3, 'a', 'b'].filter(|v| v.is_num)"# = vec![1, 2, 3].into())
            map_filter(r#"{1, 2, 3, 'a', 'b'}.filter(|k, v| v.is_num)"# = IndexMap::from([(1, 1), (2, 2), (3, 3)]).into())
            map_filter_map(r#"{1, 2, 3, 'a', 'b'}.filter { |k, v| v.is_num }.map(|k, v| (k, v * v))"# = IndexMap::from([(1, 1), (2, 4), (3, 9)]).into())
            // todo explicit tuple shouldn't be required for map function
            map_map_if(r#"{1, 2, 3, 'a', 'b'}.map(|k, v| (k, k * v) if k.is_num && v.is_num)"# = IndexMap::from([(1, 1), (2, 4), (3, 9)]).into())
            map_map(r#"{1, 2, 3}.map(|k, v| (k, k * v))"# = IndexMap::from([(1, 1), (2, 4), (3, 9)]).into())
            // self_fib_recursive(r#"
            // fn Number.fib -> Number
            //     if self <= 1
            //         self
            //     else
            //         (self - 1).fib + (self - 2).fib
            //     end
            // end
            // 6.fib
            // "# = 8.into())
        }
    }

    mod debug {
        use super::*;
        run_debug_vm! {
            // list_map_filter(r#"[1, 2, 3, 'a', 'b'].filter { |v| v.is_num }.map(|v| v * v)"# = vec![1, 4, 9].into())
            fn_calls_fn(r#"
            fn Any.apply(func: |Any| -> Any) -> List
                = func self

            3.apply { |v| v * v }"# = 9.into())
            list_map(r#"[1, 2, 3].map(|a| a * a)"# = vec![1, 4, 9].into())
            // fib_recursive(r#"
            // fn fib(n: Number) -> Number
            //     if n <= 1
            //         n
            //     else
            //         (fib n - 1) + (fib n - 2)
            //     end
            // end
            // fib 6
            // "# = 8.into())
            // map_filter_reduce(r#"
            //     [1, 37, '4', 'a'].filter { |v| v.is_num }.map { |v| v.to_i }.reduce 0, { |res, next| res += next; res }
            // "# = Value::List(vec![1.into(), 37.into(), "4".into()]))
        }
    }
}
