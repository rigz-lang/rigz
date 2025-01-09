mod vm_test {
    use rigz_vm::{
        BinaryOperation, Instruction, Lifecycle, LoadValue, Module, Number, RigzArgs, RigzBuilder,
        RigzType, Scope, TestLifecycle, TestResults, VMBuilder, VMError, Value, VM,
    };
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test(unsupported = test)]
    fn load_works() {
        let mut builder = VMBuilder::new();
        builder.add_load_instruction(Value::Number(Number::Int(42)).into());
        let mut vm = builder.build();
        let v = vm.eval().unwrap();
        assert_eq!(v, 42.into());
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn cast_works() {
        let mut builder = VMBuilder::new();
        builder
            .add_load_instruction(Value::Number(Number::Int(42)).into())
            .add_cast_instruction(RigzType::String);
        let mut vm = builder.build();
        let v = vm.eval().unwrap();
        assert_eq!(v, 42.to_string().into());
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn add_works() {
        let mut builder = VMBuilder::new();
        builder
            .add_load_instruction(42.into())
            .add_load_instruction(42.into())
            .add_add_instruction();
        let mut vm = builder.build();
        let v = vm.eval().unwrap();
        assert_eq!(v, Value::Number(Number::Int(84)).into());
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn shr_works_str_number() {
        let mut builder = VMBuilder::new();
        builder
            .add_load_instruction("abc".into())
            .add_load_instruction(1.into())
            .add_shr_instruction();
        let mut vm = builder.build();
        let v = vm.eval().unwrap();
        assert_eq!(v, "ab".into());
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn shl_works_str_number() {
        let mut builder = VMBuilder::new();
        builder
            .add_load_instruction("abc".into())
            .add_load_instruction(1.into())
            .add_shl_instruction();
        let mut vm = builder.build();
        let v = vm.eval().unwrap();
        assert_eq!(v, "bc".into());
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn call_works() {
        let mut builder = VMBuilder::new();
        let scope = builder
            .add_load_instruction("abc".into())
            .enter_scope("test", vec![], None);
        builder.exit_scope(0).add_call_instruction(scope);
        let mut vm = builder.build();
        let v = vm.eval().unwrap();
        assert_eq!(v, "abc".into());
    }

    #[derive(Copy, Clone, Debug)]
    struct TestModule {}

    #[allow(unused_variables)]
    impl<'vm> Module<'vm> for TestModule {
        fn name(&self) -> &'static str {
            "test"
        }

        fn call(&self, function: &'vm str, args: RigzArgs) -> Result<Value, VMError> {
            match function {
                "hello" => {
                    println!("{}", Value::List(args.into()));
                    Ok(Value::None)
                }
                f => Err(VMError::InvalidModuleFunction(f.to_string())),
            }
        }

        fn trait_definition(&self) -> &'static str {
            r#"
            trait test
                fn hello(var arg)
            end
            "#
        }
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn module_works() {
        let mut builder = VMBuilder::new();
        let module = TestModule {};
        builder
            .register_module(module)
            .add_load_instruction("abc".into())
            .add_call_module_instruction("test", "hello", 1);
        let mut vm = builder.build();
        let v = vm.eval().unwrap();
        assert_eq!(v, Value::None.into());
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn assignment_scopes_work() {
        let mut builder = VMBuilder::new();
        // a = 1 + 2; a + 2
        let scope = builder.enter_scope("test", vec![], None);
        builder
            .add_load_instruction(1.into())
            .add_load_instruction(2.into())
            .add_add_instruction()
            .exit_scope(0)
            .add_load_instruction(LoadValue::ScopeId(scope))
            .add_load_let_instruction("a")
            .add_get_variable_instruction("a")
            .add_load_instruction(2.into())
            .add_add_instruction()
            .add_halt_instruction();

        let mut vm = builder.build();
        let v = vm.eval().unwrap();
        assert_eq!(v, 5.into())
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn simple_scope() {
        let mut builder = VMBuilder::new();
        let scope = builder.enter_scope("test", vec![], None);
        builder
            .add_load_instruction("hello".into())
            .exit_scope(0)
            .add_load_instruction(LoadValue::ScopeId(scope))
            .add_halt_instruction();
        let mut vm = builder.build();
        assert_eq!(vm.eval().unwrap(), Value::String("hello".to_string()))
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn function_scope() {
        let mut builder = VMBuilder::new();
        let scope = builder.enter_scope("test", vec![], None);
        builder
            .add_binary_instruction(BinaryOperation::Add)
            .exit_scope(0)
            .add_load_instruction(1.into())
            .add_load_instruction(2.into())
            .add_call_instruction(scope)
            .add_load_instruction(3.into())
            .add_call_instruction(scope)
            .add_load_instruction(4.into())
            .add_call_instruction(scope)
            .add_halt_instruction();
        let mut vm = builder.build();
        assert_eq!(vm.eval().unwrap(), 10.into())
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn mutable_bin_assign() {
        let mut builder = VMBuilder::new();
        builder
            .add_load_instruction(3.into())
            .add_load_mut_instruction("a")
            .add_get_mutable_variable_instruction("a")
            .add_load_instruction(7.into())
            .add_binary_assign_instruction(BinaryOperation::Add)
            .add_get_mutable_variable_instruction("a")
            .add_halt_instruction();
        let mut vm = builder.build();
        assert_eq!(vm.eval().unwrap(), 10.into())
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn multi_mut_scope() {
        let mut vm = VM::from_scopes(vec![
            Scope {
                instructions: vec![
                    Instruction::Load(2.into()),
                    Instruction::LoadMut("a"),
                    Instruction::GetMutableVariable("a"),
                    Instruction::Call(1),
                    Instruction::Call(1),
                    Instruction::Call(1),
                    Instruction::GetMutableVariable("a"),
                    Instruction::Halt,
                ],
                ..Default::default()
            },
            Scope {
                instructions: vec![
                    Instruction::GetMutableVariable("self"),
                    Instruction::Load(3.into()),
                    Instruction::BinaryAssign(BinaryOperation::Mul),
                    Instruction::GetMutableVariable("self"),
                    Instruction::Ret,
                ],
                set_self: Some(true),
                ..Default::default()
            },
        ]);
        assert_eq!(vm.run(), 54.into(), "Run Failed {vm:#?}");
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn multi_mut_scope_get_var_between() {
        let mut vm = VM::from_scopes(vec![
            Scope {
                instructions: vec![
                    Instruction::Load(4.2.into()),
                    Instruction::LoadMut("f"),
                    Instruction::GetMutableVariable("f"),
                    Instruction::Call(1),
                    Instruction::GetMutableVariable("f"),
                    Instruction::Call(1),
                    Instruction::GetMutableVariable("f"),
                    Instruction::Call(1),
                    Instruction::GetVariable("f"),
                    Instruction::Halt,
                ],
                ..Default::default()
            },
            Scope {
                instructions: vec![
                    Instruction::GetMutableVariable("self"),
                    Instruction::Load(3.into()),
                    Instruction::BinaryAssign(BinaryOperation::Mul),
                    Instruction::GetMutableVariable("self"),
                    Instruction::Ret,
                ],
                set_self: Some(true),
                ..Default::default()
            },
        ]);
        assert_eq!(vm.run(), 113.4.into(), "Run Failed {vm:#?}")
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn test_works() {
        let mut vm = VM::from_scopes(vec![
            Scope {
                instructions: vec![Instruction::Call(2), Instruction::Halt],
                ..Default::default()
            },
            Scope {
                instructions: vec![Instruction::Load(42.into()), Instruction::Ret],
                ..Default::default()
            },
            Scope {
                instructions: vec![
                    Instruction::Load(41.into()),
                    Instruction::Call(1),
                    Instruction::Load("".into()),
                    Instruction::CallModule {
                        module: "Std",
                        func: "assert_eq",
                        args: 3,
                    },
                    Instruction::Ret,
                ],
                named: "test",
                lifecycle: Some(Lifecycle::Test(TestLifecycle)),
                args: Vec::new(),
                set_self: None,
            },
        ]);
        assert_eq!(
            vm.test(),
            TestResults {
                passed: 0,
                failed: 1,
                failure_messages: vec![("test".into(), VMError::InvalidModule("Std".to_string()))],
                duration: Default::default(),
            }
        )
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn for_list() {
        let mut builder = VMBuilder::new();
        // [for v in [1, 2, 3]: v * v]
        let scope = builder
            .add_load_instruction(vec![1, 2, 3].into())
            .enter_scope("for-list", vec![("v", false)], None);
        builder
            .add_get_variable_instruction("v")
            .add_get_variable_instruction("v")
            .add_mul_instruction()
            .exit_scope(0)
            .add_for_list_instruction(scope)
            .add_halt_instruction();
        let mut vm = builder.build();
        assert_eq!(vm.run(), vec![1, 4, 9].into())
    }
}
