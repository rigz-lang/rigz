use rigz_ast::*;

macro_rules! test_parse {
    ($($name:ident $input:literal = $expected:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let input = $input;
                let v = parse(input);
                assert_eq!(v, Ok($expected), "Failed to parse input: {}", input)
            }
        )*
    };
}

macro_rules! test_parse_equivalent {
    ($(
        $($name:ident $input:literal)*
        = $expected:expr;
    )*) => {
        $(
            $(
                #[test]
                fn $name() {
                    let input = $input;
                    let v = parse(input);
                    assert_eq!(v, Ok($expected), "Failed to parse input: {}", input)
                }
            )*
        )*
    };
}

macro_rules! test_parse_valid {
    ($($name:ident $input:literal,)*) => {
        $(
            #[test]
            fn $name() {
                let input = $input;
                let v = parse(input);
                assert_eq!(v.is_ok(), true, "Parse Failed {:?} - {}", v.unwrap_err(), input);
            }
        )*
    };
}

macro_rules! test_parse_invalid {
    ($($name:ident $input:literal,)*) => {
        $(
            #[test]
            fn $name() {
                let input = $input;
                let v = parse(input);
                assert_eq!(v.is_err(), true, "Successfully parsed invalid input {}", input);
            }
        )*
    };
}

mod invalid {
    use super::*;

    test_parse_invalid!(
        invalid_bin_exp "1 +",
        invalid_function "fn hello {}",
        let_reserved "let = 1",
        mut_reserved "mut = 1",
        end_reserved "end = 1",
        unless_reserved "unless = 1",
        if_reserved "if = 1",
        else_reserved "else = 1",
        fn_reserved "fn = 1",
        fn_call_with_parens "foo(1, 2, 3)",
    );
}

mod valid {
    use super::*;

    test_parse_valid!(
        do_one_line "do = 1 + 2",
        valid_bin_exp "1 + 2",
        valid_function "fn hello = none",
        valid_function_dollar_sign "fn $ = none",
        outer_paren_func "(foo 1, 2, 3)",
        //todo named_args_in_func "foo a: 1, b: 2, c: 3",
        let_works "let a = 1",
        mut_works "mut a = 1",
        // todo map_key_equals_values "a = {1, '2', true, none}",
        inline_unless_works "a = b unless c",
        instance_methods "a.b.c.d 1, 2, 3",
        error_def r#"
        fn error(template: String, var args) -> None
            log :error, template, args
        end
        "#,
        function_def r#"
        fn say(message: String) -> None
            puts message
        end"#,
        // todo
        unless_works r#"
            unless c
                c = 42
            end
        "#,
        list_string r#"
            let s: [String] = ["1", "a"]
        "#,
        map_string r#"
            let m: {String} = { a = "1", b = "a"}
        "#,
        // if_else_root_return r#"
        //     if c
        //         return c * 42
        //     else
        //         c = 24
        //     end
        //     c * 37
        // "#,
    );
}

test_parse_equivalent! {
    define_function_typed_oneish_line r#"
            fn hello -> String
                = "hi there"
            hello"#
    define_function_typed r#"
            fn hello -> String
                "hi there"
            end
            hello"# = Program {
            elements: vec![
                Element::Statement(Statement::FunctionDefinition(FunctionDefinition {
                    name: "hello",
                    type_definition: FunctionSignature {
                        arguments: vec![],
                        positional: true,
                        return_type: FunctionType::new(RigzType::String),
                        self_type: None,
                        var_args_start: None
                    },
                    body: Scope {
                     elements: vec![
                        Element::Expression(Expression::Value(Value::String("hi there".to_string())))
                    ],
                    },
                lifecycle: None
                })),
                Element::Expression(Expression::Identifier("hello"))
            ]
        };
    define_function r#"
            fn hello
                "hi there"
            end
            hello"#
    define_function_oneishline r#"
            fn hello
                = "hi there"
            hello"#
    define_function_oneline r#"
            fn hello = "hi there"
            hello"# = Program {
            elements: vec![
                Element::Statement(Statement::FunctionDefinition(FunctionDefinition {
                    name: "hello",
                    type_definition: FunctionSignature {
                        arguments: vec![],
                        positional: true,
                        return_type: FunctionType::new(RigzType::default()),
                        self_type: None,
                        var_args_start: None
                    },
                    body: Scope {
                    elements: vec![
                        Element::Expression(Expression::Value(Value::String("hi there".to_string())))
                    ],
                        },
                lifecycle: None
                })),
                Element::Expression(Expression::Identifier("hello"))
            ]
        };
}

test_parse! {
    symbols "foo :hello" = Program {
        elements: vec![
            Element::Expression(Expression::FunctionCall("foo", vec![Expression::Symbol("hello")]))
        ]
    },
    traits r#"trait Hello
            fn foo

            fn mut String.bar

            fn say(message: String) -> None
                puts message
            end
        end"# = Program {
        elements: vec![
            Element::Statement(Statement::Trait(TraitDefinition {
                name: "Hello",
                functions: vec![
                    FunctionDeclaration::Declaration {
                        name: "foo",
                        type_definition: FunctionSignature {
                            arguments: vec![],
                            return_type: FunctionType::new(RigzType::default()),
                            self_type: None,
                            positional: true,
                            var_args_start: None
                        },
                    },
                    FunctionDeclaration::Declaration {
                        name: "bar",
                        type_definition: FunctionSignature {
                            arguments: vec![],
                            return_type: FunctionType::mutable(RigzType::This),
                            self_type: Some(FunctionType::mutable(RigzType::String)),
                            positional: true,
                            var_args_start: None
                        },
                    },
                    FunctionDeclaration::Definition(FunctionDefinition {
                        name: "say",
                        type_definition: FunctionSignature {
                            arguments: vec![
                                FunctionArgument {
                                    name: "message",
                                    default: None,
                                    function_type: FunctionType::new(RigzType::String),
                                    var_arg: false
                                }
                            ],
                            return_type: FunctionType::new(RigzType::None),
                            self_type: None,
                            positional: true,
                            var_args_start: None
                        },
                        body: Scope {
                            elements: vec![
                                Element::Expression(Expression::FunctionCall("puts", vec!["message".into()]))
                            ]
                        },
                        lifecycle: None
                 }),
                ],
            }))
        ]
    },
    basic "1 + 2" = Program {
        elements: vec![
            Element::Expression(
                Expression::BinExp(
                    Box::new(Expression::Value(Value::Number(1.into()))),
                    BinaryOperation::Add,
                    Box::new(Expression::Value(Value::Number(2.into())))
                )
            )
        ]
    },
    complex "1 + 2 * 3" = Program {
        elements: vec![
            Element::Expression(
                Expression::BinExp(
                    Box::new(Expression::binary(
                        Expression::Value(Value::Number(1.into())),
                        BinaryOperation::Add,
                        Expression::Value(Value::Number(2.into()))
                    )),
                    BinaryOperation::Mul,
                    Box::new(Expression::Value(Value::Number(3.into())))
                )
            )
        ]
    },
    complex_parens "1 + (2 * 3)" = Program {
        elements: vec![
            Expression::binary(
                Expression::Value(Value::Number(1.into())),
                BinaryOperation::Add,
                Expression::binary(
                    Expression::Value(Value::Number(2.into())),
                    BinaryOperation::Mul,
                    Expression::Value(Value::Number(3.into()))
                )
            ).into(),
        ]
    },
    list "[1, '2', {a = 3}]" = Program {
        elements: vec![
            Element::Expression(
                Expression::List(
                    vec![
                        Expression::Value(Value::Number(1.into())),
                        Expression::Value(Value::String("2".to_string())),
                        Expression::Map(vec![(Expression::Identifier("a"), Expression::Value(Value::Number(3.into())))]),
                    ]
                )
            )
        ]
    },
    assign "a = 7 - 0" = Program {
        elements: vec![
            Element::Statement(Statement::Assignment {
                lhs: Assign::Identifier("a", false),
                expression: Expression::BinExp(
                    Box::new(Expression::Value(Value::Number(7.into()))),
                    BinaryOperation::Sub,
                    Box::new(Expression::Value(Value::Number(0.into())))
                ),
            })
        ]
    },
    define_function_args r#"
            fn add(a, b, c)
              a + b + c
            end
            add 1, 2, 3"# = Program {
        elements: vec![
            Element::Statement(Statement::FunctionDefinition(FunctionDefinition {
                name: "add",
                type_definition: FunctionSignature {
                    positional: true,
                    arguments: vec![
                        FunctionArgument {
                            name: "a",
                            default: None,
                            function_type: RigzType::Any.into(),
                            var_arg: false
                        },
                        FunctionArgument {
                            name: "b",
                            default: None,
                            function_type: RigzType::Any.into(),
                            var_arg: false
                        },
                        FunctionArgument {
                            name: "c",
                            default: None,
                            function_type: RigzType::Any.into(),
                            var_arg: false
                        },
                    ],
                    return_type: FunctionType::new(RigzType::default()),
                    self_type: None,
                    var_args_start: None
                },
                body: Scope {
                    elements: vec![
                        Expression::binary(
                            Expression::binary("a".into(), BinaryOperation::Add, "b".into()),
                            BinaryOperation::Add,
                            "c".into()
                        ).into(),
                    ],
                },
                lifecycle: None
            })),
            Element::Expression(Expression::FunctionCall("add", vec![Expression::Value(Value::Number(1.into())), Expression::Value(Value::Number(2.into())), Expression::Value(Value::Number(3.into()))]))
        ]
    },
    multi_complex_parens "1 + (2 * (2 - 4)) / 4" = Program {
        elements: vec![
            Element::Expression(
                Expression::BinExp(
                    Box::new(Expression::BinExp(
                    Box::new(Expression::Value(Value::Number(1.into()))),
                    BinaryOperation::Add,
                    Box::new(Expression::BinExp(
                        Box::new(Expression::Value(Value::Number(2.into()))),
                        BinaryOperation::Mul,
                        Box::new(Expression::BinExp(
                                Box::new(Expression::Value(Value::Number(2.into()))),
                                BinaryOperation::Sub,
                                Box::new(Expression::Value(Value::Number(4.into()))))
                            ))
                        )
                    )
                ),
                    BinaryOperation::Div,
                    Box::new(
                        Expression::Value(Value::Number(4.into()))
                    )
                )
            )
        ]
    },
    union_type "a: String | Number | Bool = false" = Program {
        elements: vec![
            Statement::Assignment {
                lhs: Assign::TypedIdentifier("a", false, RigzType::Union(vec![RigzType::String, RigzType::Number, RigzType::Bool])),
                expression: Expression::Value(false.into()),
            }.into()
        ],
    },
    composite_type r#"
        type Foo = {
            foo: Number
        }
        type Bar = {
            bar: Number
        }
        a: Foo & Bar = { foo = 1, bar = 7}
    "# = Program {
        elements: vec![
            Statement::TypeDefinition("Foo", RigzType::Custom(CustomType {
                name: "Foo".to_string(),
                fields: vec![
                    ("foo".into(), RigzType::Number)
                ],
            })).into(),
            Statement::TypeDefinition("Bar", RigzType::Custom(CustomType {
                name: "Bar".to_string(),
                fields: vec![
                    ("bar".into(), RigzType::Number)
                ],
            })).into(),
            Statement::Assignment {
                lhs: Assign::TypedIdentifier("a", false, RigzType::Composite(vec![RigzType::Custom(CustomType {
                    name: "Foo".to_string(),
                    fields: vec![],
                }), RigzType::Custom(CustomType {
                    name: "Bar".to_string(),
                    fields: vec![],
                })])),
                expression: Expression::Map(vec![
                    (Expression::Identifier("foo"), Expression::Value(1.into())),
                    (Expression::Identifier("bar"), Expression::Value(7.into())),
                ])
            }.into()
        ],
    },
    union_composite_type_parens r#"
        type Message = { message: String }
        type Id = { id: Number }
        type Result = String | (Message & Id)
        mut s: Result = ""
    "# = Program {
        elements: vec![
            Statement::TypeDefinition("Message", RigzType::Custom(CustomType {
                name: "Message".to_string(),
                fields: vec![
                    ("message".into(), RigzType::String)
                ],
            })).into(),
            Statement::TypeDefinition("Id", RigzType::Custom(CustomType {
                name: "Id".to_string(),
                fields: vec![
                    ("id".into(), RigzType::Number)
                ],
            })).into(),
            Statement::TypeDefinition("Result", RigzType::Union(vec![
                RigzType::String, RigzType::Composite(vec![RigzType::Custom(CustomType {
                    name: "Message".to_string(),
                    fields: vec![],
                }), RigzType::Custom(CustomType {
                    name: "Id".to_string(),
                    fields: vec![],
                })])
            ])).into(),
            Statement::Assignment {
                lhs: Assign::TypedIdentifier("s", true, RigzType::Custom(CustomType {
                    name: "Result".to_string(),
                    fields: vec![],
                })),
                expression: Expression::Value("".into())
            }.into()
        ],
    },
    // todo support later
    // define_function_named_args r#"
    //     fn add{a, b, c}
    //       a + b + c
    //     end
    //     v = {a = 1, b = 2, c = 3}
    //     add v"# = Program {
    //     elements: vec![
    //         Element::Statement(Statement::FunctionDefinition {
    //             name: "add",
    //             type_definition: FunctionDefinition {
    //                 positional: false,
    //                 arguments: vec![
    //                     FunctionArgument {
    //                         name: Some("a"),
    //                         default: None,
    //                         rigz_type: RigzType::Any,
    //                     },
    //                     FunctionArgument {
    //                         name: Some("b"),
    //                         default: None,
    //                         rigz_type: RigzType::Any,
    //                     },
    //                     FunctionArgument {
    //                         name: Some("c"),
    //                         default: None,
    //                         rigz_type: RigzType::Any,
    //                     },
    //                 ],
    //                 return_type: RigzType::Any
    //             },
    //             elements: vec![
    //                 Element::Expression(Expression::BinExp(
    //                     Box::new(Expression::Identifier("a")),
    //                     BinaryOperation::Add,
    //                     Box::new(Expression::BinExp(
    //                             Box::new(Expression::Identifier("b")),
    //                             BinaryOperation::Add,
    //                             Box::new(Expression::Identifier("c")))
    //                 )))                    ],
    //         }),
    //         Element::Statement(Statement::Assignment {
    //             name: "v",
    //             mutable: false,
    //             expression: Expression::Map(vec![(Expression::Identifier("a"), Expression::Value(Value::Number(1.into()))), (Expression::Identifier("b"), Expression::Value(Value::Number(2.into()))), (Expression::Identifier("c"), Expression::Value(Value::Number(3.into())))]),
    //         }),
    //         Element::Expression(Expression::FunctionCall("add", vec![Expression::Identifier("v")]))
    //     ]
    // },
}

// mod debug {
//     use super::*;
//
//     test_parse! {}
// }