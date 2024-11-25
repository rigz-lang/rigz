mod vm_test {
    use rigz_vm::{
        BinaryAssign, BinaryOperation, Instruction, Lifecycle, Module, Number, RegisterValue,
        RigzArgs, RigzBuilder, RigzType, Scope, TestLifecycle, TestResults, VMBuilder, VMError,
        Value, VM,
    };
    use std::str::FromStr;

    #[test]
    fn load_works() {
        let mut builder = VMBuilder::new();
        builder.add_load_instruction(4, Value::Number(Number::Int(42)).into());
        let mut vm = builder.build();
        vm.eval().unwrap();
        assert_eq!(vm.get_register(4), Value::Number(Number::Int(42)).into());
    }

    #[test]
    fn cast_works() {
        let mut builder = VMBuilder::new();
        builder
            .add_load_instruction(4, Value::Number(Number::Int(42)).into())
            .add_cast_instruction(4, RigzType::String, 7);
        let mut vm = builder.build();
        vm.eval().unwrap();
        assert_eq!(vm.get_register(7), Value::String(42.to_string()).into());
    }

    #[test]
    fn add_works() {
        let mut builder = VMBuilder::new();
        builder
            .add_load_instruction(4, Value::Number(Number::Int(42)).into())
            .add_copy_instruction(4, 37)
            .add_add_instruction(4, 37, 82);
        let mut vm = builder.build();
        vm.eval().unwrap();
        assert_eq!(vm.get_register(82), Value::Number(Number::Int(84)).into());
    }

    #[test]
    fn copy_works() {
        let mut builder = VMBuilder::new();
        builder
            .add_load_instruction(4, Value::Number(Number::Int(42)).into())
            .add_copy_instruction(4, 37);
        let mut vm = builder.build();
        vm.eval().unwrap();
        assert_eq!(vm.get_register(37), Value::Number(Number::Int(42)).into());
    }

    #[test]
    fn shr_works_str_number() {
        let mut builder = VMBuilder::new();
        builder
            .add_load_instruction(2, Value::String(String::from_str("abc").unwrap()).into())
            .add_load_instruction(3, Value::Number(Number::Int(1)).into())
            .add_shr_instruction(2, 3, 4);
        let mut vm = builder.build();
        vm.eval().unwrap();
        assert_eq!(
            vm.get_register(4),
            Value::String(String::from_str("ab").unwrap()).into()
        );
    }

    #[test]
    fn shl_works_str_number() {
        let mut builder = VMBuilder::new();
        builder
            .add_load_instruction(2, Value::String(String::from_str("abc").unwrap()).into())
            .add_load_instruction(3, Value::Number(Number::Int(1)).into())
            .add_shl_instruction(2, 3, 4);
        let mut vm = builder.build();
        vm.eval().unwrap();
        assert_eq!(
            vm.get_register(4),
            Value::String(String::from_str("bc").unwrap()).into()
        );
    }

    #[test]
    fn call_works() {
        let mut builder = VMBuilder::new();
        let scope = builder
            .add_load_instruction(2, Value::String(String::from_str("abc").unwrap()).into())
            .enter_scope("test", vec![]);
        builder
            .add_copy_instruction(2, 3)
            .exit_scope(0, 3)
            .add_call_instruction(scope, vec![], 3);
        let mut vm = builder.build();
        vm.eval().unwrap();
        assert_eq!(
            vm.get_register(3),
            Value::String(String::from_str("abc").unwrap()).into()
        );
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

    #[test]
    fn module_works<'vm>() {
        let mut builder = VMBuilder::new();
        let module = TestModule {};
        builder
            .register_module(module)
            .add_load_instruction(2, Value::String(String::from_str("abc").unwrap()).into())
            .add_call_module_instruction("test", "hello", vec![2], 3);
        let mut vm = builder.build();
        vm.eval().unwrap();
        assert_eq!(vm.get_register(3), Value::None.into());
    }

    #[test]
    fn assignment_scopes_work() {
        pretty_env_logger::init();
        let mut builder = VMBuilder::new();
        // a = 1 + 2; a + 2
        let scope = builder.enter_scope("test", vec![]);
        builder
            .add_load_instruction(2, Value::Number(Number::Int(1)).into())
            .add_load_instruction(3, Value::Number(Number::Int(2)).into())
            .add_add_instruction(2, 3, 4)
            .exit_scope(0, 4)
            .add_load_instruction(5, RegisterValue::ScopeId(scope, vec![], 4))
            .add_load_let_instruction("a", 5)
            .add_get_variable_instruction("a", 6)
            .add_load_instruction(7, Value::Number(Number::Int(2)).into())
            .add_add_instruction(6, 7, 8)
            .add_halt_instruction(8);

        let mut vm = builder.build();
        let v = vm.eval().unwrap();
        assert_eq!(v, Value::Number(Number::Int(5)))
    }

    #[test]
    fn simple_scope() {
        let mut builder = VMBuilder::new();
        let scope = builder.enter_scope("test", vec![]);
        builder
            .add_load_instruction(2, Value::String("hello".to_string()).into())
            .exit_scope(0, 2)
            .add_load_instruction(4, RegisterValue::ScopeId(scope, vec![], 2))
            .add_halt_instruction(4);
        let mut vm = builder.build();
        assert_eq!(vm.eval().unwrap(), Value::String("hello".to_string()))
    }

    #[test]
    fn function_scope() {
        let mut builder = VMBuilder::new();
        let scope = builder.enter_scope("test", vec![]);
        builder
            .add_binary_instruction(BinaryOperation::Add, 1, 2, 3)
            .exit_scope(0, 3)
            .add_load_instruction(1, RegisterValue::Value(1.into()))
            .add_load_instruction(2, RegisterValue::Value(2.into()))
            .add_call_instruction(scope, vec![], 3)
            .add_load_instruction(1, RegisterValue::Register(3))
            .add_load_instruction(2, RegisterValue::Value(3.into()))
            .add_call_instruction(scope, vec![], 3)
            .add_load_instruction(1, RegisterValue::Register(3))
            .add_load_instruction(2, RegisterValue::Value(4.into()))
            .add_call_instruction(scope, vec![], 3)
            .add_halt_instruction(3);
        let mut vm = builder.build();
        assert_eq!(vm.eval().unwrap(), 10.into())
    }

    #[test]
    fn mutable_bin_assign() {
        let mut builder = VMBuilder::new();
        builder
            .add_load_instruction(1, 3.into())
            .add_load_instruction(2, 7.into())
            .add_load_mut_instruction("a", 1)
            .add_binary_assign_instruction(BinaryOperation::Add, 1, 2)
            .add_halt_instruction(1);
        let mut vm = builder.build();
        assert_eq!(vm.eval().unwrap(), 10.into())
    }

    #[test]
    fn mutable_get_var_assign() {
        let mut builder = VMBuilder::new();
        builder
            .add_load_instruction(1, 3.into())
            .add_load_instruction(2, 7.into())
            .add_load_mut_instruction("a", 1)
            .add_get_mutable_variable_instruction("a", 4)
            .add_binary_assign_instruction(BinaryOperation::Add, 4, 2)
            .add_halt_instruction(1);
        let mut vm = builder.build();
        assert_eq!(vm.eval().unwrap(), 10.into())
    }

    #[test]
    fn multi_mut_scope() {
        let mut vm = VM {
            scopes: vec![
                Scope {
                    instructions: vec![
                        Instruction::Load(89, RegisterValue::Value(2.into())),
                        Instruction::LoadMutRegister("a", 89),
                        Instruction::GetMutableVariable("a", 85),
                        Instruction::CallSelf {
                            scope: 1,
                            args: vec![],
                            this: 85,
                            output: 85,
                            mutable: true,
                        },
                        Instruction::CallSelf {
                            scope: 1,
                            args: vec![],
                            this: 85,
                            output: 85,
                            mutable: true,
                        },
                        Instruction::CallSelf {
                            scope: 1,
                            args: vec![],
                            this: 85,
                            output: 85,
                            mutable: true,
                        },
                        // GetVariable creates a copy
                        Instruction::GetMutableVariable("a", 90),
                        Instruction::Halt(90),
                    ],
                    ..Default::default()
                },
                Scope {
                    instructions: vec![
                        Instruction::GetSelf(86, true),
                        Instruction::Load(87, RegisterValue::Value(3.into())),
                        Instruction::BinaryAssign(BinaryAssign {
                            op: BinaryOperation::Mul,
                            lhs: 86,
                            rhs: 87,
                        }),
                        Instruction::GetSelf(88, true),
                        Instruction::Load(85, RegisterValue::Register(88)),
                        Instruction::Ret(85),
                    ],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        assert_eq!(vm.run(), 54.into(), "Run Failed {vm:#?}");
        let current = vm.current.borrow();
        let results: Vec<_> = current
            .registers
            .iter()
            .filter(|(_, v)| {
                let b = v.borrow();
                b.clone() == RegisterValue::Value(54.into())
            })
            .map(|(i, _)| i)
            .collect();
        assert_eq!(1, results.len(), "Multiple matches - {results:?}");
    }

    #[test]
    fn multi_mut_scope_get_var_between() {
        let mut vm = VM {
            scopes: vec![
                Scope {
                    instructions: vec![
                        Instruction::Load(89, RegisterValue::Value(4.2.into())),
                        Instruction::LoadMutRegister("f", 89),
                        Instruction::GetMutableVariable("f", 85),
                        Instruction::CallSelf {
                            scope: 1,
                            args: vec![],
                            this: 85,
                            output: 85,
                            mutable: true,
                        },
                        Instruction::GetMutableVariable("f", 85),
                        Instruction::CallSelf {
                            scope: 1,
                            args: vec![],
                            this: 85,
                            output: 85,
                            mutable: true,
                        },
                        Instruction::GetMutableVariable("f", 85),
                        Instruction::CallSelf {
                            scope: 1,
                            args: vec![],
                            this: 85,
                            output: 85,
                            mutable: true,
                        },
                        Instruction::GetVariable("f", 90),
                        Instruction::Halt(90),
                    ],
                    ..Default::default()
                },
                Scope {
                    instructions: vec![
                        Instruction::GetSelf(86, true),
                        Instruction::Load(87, RegisterValue::Value(3.into())),
                        Instruction::BinaryAssign(BinaryAssign {
                            op: BinaryOperation::Mul,
                            lhs: 86,
                            rhs: 87,
                        }),
                        Instruction::GetSelf(88, true),
                        Instruction::Load(85, RegisterValue::Register(88)),
                        Instruction::Ret(85),
                    ],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        assert_eq!(vm.run(), 113.4.into(), "Run Failed {vm:#?}")
    }

    #[test]
    fn test_works() {
        let mut vm = VM {
            scopes: vec![
                Scope {
                    instructions: vec![
                        Instruction::Call {
                            scope: 2,
                            output: 2,
                            args: vec![],
                        },
                        Instruction::Move(2, 100),
                        Instruction::Halt(100),
                    ],
                    ..Default::default()
                },
                Scope {
                    instructions: vec![
                        Instruction::Load(1, RegisterValue::Value(42.into())),
                        Instruction::Load(0, RegisterValue::Register(1)),
                        Instruction::Ret(0),
                    ],
                    ..Default::default()
                },
                Scope {
                    instructions: vec![
                        Instruction::Load(96, RegisterValue::Value(41.into())),
                        Instruction::Load(82, RegisterValue::Register(96)),
                        Instruction::Call {
                            scope: 1,
                            output: 0,
                            args: vec![],
                        },
                        Instruction::Move(0, 97),
                        Instruction::Load(83, RegisterValue::Register(97)),
                        Instruction::Load(98, RegisterValue::Value("".into())),
                        Instruction::Load(84, RegisterValue::Register(98)),
                        Instruction::CallModule {
                            module: "Std",
                            func: "assert_eq",
                            args: vec![82, 83, 84],
                            output: 99,
                        },
                        Instruction::Load(2, RegisterValue::Register(99)),
                        Instruction::Ret(2),
                    ],
                    named: "test",
                    lifecycle: Some(Lifecycle::Test(TestLifecycle)),
                    args: Vec::new(),
                },
            ],
            ..Default::default()
        };
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
}
