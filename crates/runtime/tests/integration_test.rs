pub mod runtime {
    use rigz_core::{PrimitiveValue, VMError};
    #[allow(unused_imports)] // used by macro
    use rigz_runtime::runtime::{eval, eval_print_vm};
    use rigz_runtime::RuntimeError;
    use wasm_bindgen_test::*;

    macro_rules! run_expected {
        (ignore: $($name:ident($input:literal = $expected:expr))*) => {
            $(
                #[ignore]
                #[wasm_bindgen_test(unsupported = test)]
                fn $name() {
                    let input = $input.to_string();
                    let v = eval(input);
                    assert_eq!(v, Ok($expected.into()), "VM eval failed {}", $input);
                }
            )*
        };
        ($($name:ident($input:literal = $expected:expr))*) => {
            $(
                 #[wasm_bindgen_test(unsupported = test)]
                fn $name() {
                    let input = $input.to_string();
                    let v = eval(input);
                    assert_eq!(v, Ok($expected.into()), "VM eval failed {}", $input);
                }
            )*
        };
    }

    #[allow(unused_macros)]
    macro_rules! run_debug_vm {
        ($($name:ident($input:literal = $expected:expr))*) => {
            $(
                 #[wasm_bindgen_test(unsupported = test)]
                fn $name() {
                    let _ = pretty_env_logger::try_init();
                    let input = $input.to_string();
                    let v = eval_print_vm(input);
                    assert_eq!(v, Ok($expected.into()), "VM eval failed {}", $input);
                }
            )*
        };
    }

    macro_rules! run_error {
        ($($name:ident($input:literal = $expected:expr))*) => {
            $(
                 #[wasm_bindgen_test(unsupported = test)]
                fn $name() {
                    let input = $input.to_string();
                    let v = eval(input);
                    assert_eq!(v, Err($expected.into()), "Failed to parse input {}", $input)
                }
            )*
        };
    }

    macro_rules! run_error_starts_with {
        ($($name:ident($input:literal = $expected:literal))*) => {
            $(
                 #[wasm_bindgen_test(unsupported = test)]
                fn $name() {
                    let input = $input.to_string();
                    let v = eval(input);
                    let Err(RuntimeError::Run(VMError::RuntimeError(e))) = v else {
                        assert!(false, "Unexpected result {v:?} for {}", $input);
                        return
                    };
                    assert!(e.starts_with($expected), "Unexpected result {e:?} for {}", $input)
                }
            )*
        };
    }

    macro_rules! run_invalid {
        ($($name:ident($input:literal))*) => {
            $(
                 #[wasm_bindgen_test(unsupported = test)]
                fn $name() {
                    let input = $input.to_string();
                    let v = eval(input);
                    assert!(v.is_err(), "Successfully parsed invalid input: {}", $input)
                }
            )*
        };
    }

    pub mod invalid {
        use super::*;
        use rigz_core::VMError;

        run_invalid! {
            // last statement must be an expression
            assign("a = 3 * 2")
            var_once_in_fn_def("fn foo(var foo, var bar) = none")
        }

        run_error! {
            // todo better error message here, ideally this fails during validation
            import_required("1.to_json" = VMError::UnsupportedOperation("Cannot read to_json for 1".to_string()))
            raise_error("raise 'something went wrong'" = VMError::RuntimeError("something went wrong".to_string()))
            assert("assert_eq 1, 2" = VMError::RuntimeError("Assertion Failed\n\t\tLeft: 1\n\t\tRight: 2".to_string()))
            stack_overflow(r#"fn foo
                foo
            end
            foo
            "# = VMError::RuntimeError("Stack overflow: exceeded 1024".to_string()))
        }

        run_error_starts_with! {
            on_timeout_works(r#"
            @on("message")
            fn foo(a)
                sleep 1
                a * 2
            end

            pids = send 'message', 21
            receive pids.0, 0
            "# = "`receive` timed out after 0ms")
        }
    }

    pub mod valid {
        use super::*;
        use rigz_core::{IndexMap, ObjectValue};

        run_expected! {
            raw_value("'Hello World'" = "Hello World")
            addition("2 + 2" = 4)
            list_index("[1, 2, 3][2]" = 3)
            list_index_getter("[1, 2, 3].2" = 3)
            map_sum("{1, 2, 3}.sum" = 6)
            split_first("[1, 2, 3].split_first" = ObjectValue::Tuple(vec![1.into(), vec![2, 3].into()]))
            split_first_map("{1, 2, 3}.split_first" = ObjectValue::Tuple(vec![ObjectValue::Tuple(vec![1.into(), 1.into()].into()), ObjectValue::Map(IndexMap::from([(2.into(), 2.into()), (3.into(), 3.into())]))]))
            split_first_assign("(first, rest) = [1, 2, 3].split_first; first + rest" = vec![1, 2, 3])
            complex_expression_ignore_precedence("1 + 2 * 3 - 4 / 5" = 1)
            ignore_precedence("2 + 1 * 3" = 9)
            paren_precedence("2 + (1 * 3)" = 5)
            assign("a = 3 * 2; a" = 6)
            assign_add("a = 1 + 2; a + 2" = 5)
            mutable_add("mut a = 4; a += 2; a" = 6)
            mutable_sub("mut a = 4; a -= 2; a" = 2)
            to_s("1.to_s" = "1")
            unary_not("!1" = false)
            unary_neg("-2.5" = -2.5)
            binary_expr_function_call(r#"
            fn foo(number: Number) -> Number
                number * 2
            end
            a = 3
            1.to_s + foo a
            "# = 7)
            named_for_positional(r#"
            fn foo(bar: Number) -> Number
                bar * 24
            end
            foo bar: 3
            "# = 72)
            call_function_multiple_times(r#"
            fn foo(number: Number) -> Number
                number * 2
            end
            a = 1
            foo (foo (foo a))
            "# = 8)
            call_extension_function_mutable(r#"
            fn mut Number.foo -> mut Self
                self *= 3
                self
            end
            mut b = 2
            b.foo
            b
            "# = 6)
            call_extension_function_multiple_times_inline(r#"
            fn mut Number.foo -> mut Self
                self *= 3
                self
            end
            mut a = 2
            ((a.foo).foo).foo
            a
            "# = 54)
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
            "# = 113.4)
            instance_get(r#"
                m = {a = {b = {c = 1}}}
                m.a.b.c
            "# = 1)
            binary_expr_instance_call(r#"
            11.4 + 1.2.ceil
            "# = 13.4)
            create_list(r#"
            [1, 2, 3, 4]
            "# = vec![1, 2, 3, 4])
            create_map(r#"
            {1, 2, 3, 4}
            "# = IndexMap::from([(1, 1), (2, 2), (3, 3), (4, 4)]))
            create_dynamic_list(r#"
                [{d = 1}]
            "# = vec![ObjectValue::Map(IndexMap::from([("d".into(), 1.into())]))])
            call_extension_function_multiple_times_inline_no_parens(r#"
            fn mut String.foo -> mut Self
                self += "h"
                self
            end
            mut a = ""
            a.foo.foo.foo
            a == "hhh"
            "# = true)
            call_module_extension_function_in_extension_scope(r#"
            fn String.foo -> Self
                "h" + self.to_s
            end
            "i".foo
            "# = "hi")
            lte("6 <= 1" = false)
            gte("6 >= 1" = true)
            if_true(r#"if 0 == none
                14
            end"# = 14)
            trailing_if(r#"v = 42; v if v.is_num"# = 42)
            instance_trailing_if_in_func(r#"
                fn foo(a)
                    a.to_i if a.is_num
                end
                foo 'a'
            "# = PrimitiveValue::None)
            if_false(r#"if 1 == "abc"
                14
            end"# = PrimitiveValue::None)
            to_json("import JSON; {a=5}.to_json" = r#"{"a":5}"#)
            json_parse("import JSON; JSON.parse '5'" = 5)
            is("1.is Number" = true)
            fn_calls_fn(r#"
            fn Any.apply(func: |Any| -> Any)
                func self
            end

            3.apply { |v| v * v }"# = 9)
            fib_recursive_dynamic_programming(r#"
            @memo
            fn fib(n: Number) -> Number
                if n <= 1
                    n
                else
                    b = n - 2
                    (fib n - 1) + fib b
                end
            end
            fib 10
            "# = 55)
            if_else_true(r#"if 0 == ""
                42
            else
                37
            end"# = 42)
            if_else_false(r#"if !true
                42
            else
                1 + 2
            end"# = 3)
            var_args_module(r#"
            let a = []
            a.with 1, 2, 3
            "# = vec![1, 2, 3])
            var_args_module_str(r#"
            let a = ""
            a.with 1, 2, 3
            "# = "123")
            lambda(r#"
            a = || 42
            a
            "# = 42)
            for_list(r#"[for v in [1, 2, 3]: v * v]"# = vec![1, 4, 9])
            for_list_exclude_nones(r#"[for v in [1, 2, 3, 'a', 'b']: v if v.is_num]"# = vec![1, 2, 3])
            for_map(r#"{for k, v in {1, 2, 3}: k, v if k % 2 == 0}"# = IndexMap::from([(2, 2)]))
            lambda_in_for_list_if_expression(r#"
            func = |v| v if v.is_num
            [for a in ['a', 'b', 'c', 1, 2, 3]: func a]
            "# = vec![1, 2, 3])
            lambda_in_for_list(r#"
            func = |v| v.is_num
            [for a in ['a', 'b', 'c', 1, 2, 3]: a if func a]
            "# = vec![1, 2, 3])
            trailing_if_false(r#"v = 'a'; v if v.is_num"# = PrimitiveValue::None)
            instance_trailing_if(r#"a = 'a'; a.to_i if a.is_num"# = PrimitiveValue::None)
            filter(r#"[1, 2, 3, 'a', 'b'].filter(|v| v.is_num)"# = vec![1, 2, 3])
            map_filter(r#"{1, 2, 3, 'a', 'b'}.filter(|k, v| v.is_num)"# = IndexMap::from([(1, 1), (2, 2), (3, 3)]))
            map_filter_map(r#"{1, 2, 3, 'a', 'b'}.filter { |k, v| v.is_num }.map(|k, v| (k, v * v))"# = IndexMap::from([(1, 1), (2, 4), (3, 9)]))
            map_map_if(r#"{1, 2, 3, 'a', 'b'}.map(|k, v| (k, k * v) if k.is_num && v.is_num)"# = IndexMap::from([(1, 1), (2, 4), (3, 9)]))
            map_map(r#"{1, 2, 3}.map(|k, v| (k, k * v))"# = IndexMap::from([(1, 1), (2, 4), (3, 9)]))
            list_map_filter(r#"[1, 2, 3, 'a', 'b'].filter { |v| v.is_num }.map(|v| v * v)"# = vec![1, 4, 9])
            list_map(r#"[1, 2, 3].map(|a| a * a)"# = vec![1, 4, 9])
            self_fib_recursive(r#"
            fn Number.fib -> Number
                if self <= 1
                    self
                else
                    (self - 1).fib + (self - 2).fib
                end
            end
            6.fib
            "# = 8)
            list_dup(r#"
            mut a = [1, 2, 3]
            a.extend a
            a
            "# = vec![1, 2, 3, 1, 2, 3])
            list_dup_clone(r#"
            mut a = [1, 2, 3]
            a.extend a.clone
            a
            "# = vec![1, 2, 3, 1, 2, 3])
            list_multi_assign(r#"
            mut a = [1, 2, 3]
            a = a + a
            a
            "# = vec![1, 2, 3, 1, 2, 3])
            map_filter_reduce(r#"
                [1, 37, '4', 'a'].filter { |v| v.is_num }.map { |v| v.to_i }.reduce(0, |res, next| res + next)
            "# = 42)
            map_filter_reduce_subtract(r#"
                [1, 37, '4', 'a'].filter { |v| v.is_num }.map { |v| v.to_i }.reduce(100, |res, next| res - next)
            "# = 58)
            map_filter_reduce_func_ref(r#"
                fn foo(c, d)
                    if d.is_num
                        puts 'c', c, 'd', d
                        c + d.to_i
                    else
                        c
                    end
                end

                [1, 37, '4', 'a'].reduce(0, foo)
            "# = 42)
            list_map_if(r#"
                [1, 37, '4', 'a'].map(|v| v.to_i if v.is_num)
            "# = vec![1, 37, 4])
            trait_impl(r#"
            trait Hello
                fn Self.hello -> String
            end

            impl Hello for Any
                fn Self.hello -> String = "Hello"
            end

            1.hello
            "# = "Hello")
            early_return(r#"
            if true
                return 42
            end

            37
            "# = 42)
            func_early_return(r#"
            fn foo
                if true
                    return 42
                end
                30
            end

            foo + 37
            "# = 79)
            func_early_return_trailing(r#"
            fn foo
                return 42 unless false
                30
            end

            foo + 37
            "# = 79)
            unless_false(r#"
            a = 37 unless false
            a || 42
            "# = 37)
            unless_true(r#"
            a = unless true
                37
            end
            a || 42
            "# = 42)
            fn_acts_as_closure(r#"
            mut b = 0
            fn a
                b += 1
                7 * b
            end

            a + a
            "# = 21)
            default_args_work_modules(r#"
            import Random
            next_bool || true
            "# = true)
            default_args_can_be_overwritten(r#"
            import Random
            next_bool 1
            "# = true)
            mut_self_clone(r#"
            mut a = 2
            a += a.clone
            a
            "# = 4)
            mut_self(r#"
            mut a = 2
            a += a
            a
            "# = 4)
            mut_self_unary(r#"
            mut a = false
            a = !a
            a
            "# = true)
            scopes_run_once(r#"
            mut b = 0
            a = do
                b += 1
                7 * b
            end

            a + a
            "# = 14)
            scopes_run_once_in_fn(r#"
            mut a = 1
            bar = do
                a += 1
                21 * a
            end

            fn foo = bar

            foo / foo
            "# = 1)
            list_sum(r#"[1, 20, 21].sum"# = 42)
            puts_is_none("puts 1, 2, 3" = ())
            puts_assign("a = puts 1, 2, 3; a" = ())
            into(r#"
            mut a = []
            [1] |> a.extend
            foo = 'hi'
            [2, 3] |> a.extend
            a
            "# = vec![1, 2, 3])
            args_into(r#"
            fn add(a, b) = a + b

            puts 1, 2, 3
            |> add 6
            "# = 6)
            fn_calls_fn_two_args(r#"
            fn apply(value, func: |Any, Any| -> Any)
                func value, value - 1
            end

            apply 4, do |v, b|
                puts v, b
                v - b
             end"# = 1)
            format("format '{}', 1 + 2" = "3")
            format_parens("format('{}', 1 + 2)" = "3")
            on_works(r#"
            @on("message")
            fn foo(a) = a * 2

            pids = send 'message', 21
            receive pids.0
            "# = 42)
            on_works_multi_message(r#"
            @on("message")
            fn foo(a, b) = a * b

            @on("message")
            fn bar(a, b) = a - b

            pids = send 'message', 21, 12
            receive pids
            "# = vec![252, 9])
            to_bits(
                "2.to_bits" = vec![true, false]
            )
            from_bits(
                "int_from_bits [true, false]" = 2
            )
            spawn_works(r#"
            pid = spawn do
                42
            end

            receive pid
            "# = 42)
        }
    }

    pub mod debug {
        use super::*;
        run_debug_vm! {}
    }

    pub mod recursive {
        use super::*;
        run_expected! {
            factorial(r#"
            fn factorial(n: Number)
                if n == 0
                    1
                else
                    a = factorial n - 1
                    n * a
                end
            end
            factorial 4
            "#=24)
            factorial_inline(r#"
            fn factorial(n: Number)
                if n == 0
                    1
                else
                    n * factorial n - 1
                end
            end
            factorial 4
            "#=24)
            fib_recursive(r#"
            fn fib(n: Number) -> Number
                if n <= 1
                    n
                else
                    (fib n - 1) + (fib n - 2)
                end
            end
            fib 6
            "# = 8)
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
            "#=1307674368000_i64)
        }
    }
}
