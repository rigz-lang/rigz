extern crate core;

mod builder;
mod call_frame;
mod instructions;
mod lifecycle;
mod macros;
mod module;
mod number;
mod objects;
mod scope;
mod traits;
mod value;
mod value_range;
mod vm;

pub use builder::VMBuilder;
pub use call_frame::{CallFrame, Variable};
pub use instructions::{
    Binary, BinaryAssign, BinaryOperation, Clear, Instruction, Unary, UnaryAssign, UnaryOperation,
};
pub use module::Module;
pub use number::Number;
pub use objects::RigzType;
pub use scope::Scope;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
pub use traits::{Logical, Reverse};
pub use value::Value;
pub use value_range::ValueRange;
pub use vm::{RegisterValue, VM};

pub type Register = usize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
// Tagged to avoid confusion with string deserialization
pub enum VMError {
    RuntimeError(String),
    EmptyRegister(String),
    ConversionError(String),
    ScopeDoesNotExist(String),
    UnsupportedOperation(String),
    VariableDoesNotExist(String),
    InvalidModule(String),
    InvalidModuleFunction(String),
    LifecycleError(String),
}

impl VMError {
    pub fn to_value(self) -> Value {
        Value::Error(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::number::Number;
    use crate::value::Value;
    use crate::vm::RegisterValue;
    use crate::{BinaryAssign, BinaryOperation, Instruction, Module, RigzType, Scope, VMBuilder, VMError, VM};
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
        builder
            .add_load_instruction(2, Value::String(String::from_str("abc").unwrap()).into())
            .enter_scope()
            .add_copy_instruction(2, 3)
            .exit_scope(3)
            .add_call_instruction(1, 3);
        let mut vm = builder.build();
        vm.eval().unwrap();
        assert_eq!(
            vm.get_register(3),
            Value::String(String::from_str("abc").unwrap()).into()
        );
    }

    #[derive(Copy, Clone)]
    struct TestModule {}

    #[allow(unused_variables)]
    impl<'vm> Module<'vm> for TestModule {
        fn name(&self) -> &'static str {
            "test"
        }

        fn call(&self, function: &'vm str, args: Vec<Value>) -> Result<Value, VMError> {
            match function {
                "hello" => {
                    println!("{}", Value::List(args));
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
        assert_eq!(
            vm.registers.get(&3).unwrap().clone().into_inner(),
            Value::None.into()
        );
    }

    #[test]
    fn assignment_scopes_work() {
        pretty_env_logger::init();
        let mut builder = VMBuilder::new();
        // a = 1 + 2; a + 2
        builder
            .enter_scope()
            .add_load_instruction(2, Value::Number(Number::Int(1)).into())
            .add_load_instruction(3, Value::Number(Number::Int(2)).into())
            .add_add_instruction(2, 3, 4)
            .exit_scope(4)
            .add_load_instruction(5, RegisterValue::ScopeId(1, 4))
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
        builder
            .enter_scope()
            .add_load_instruction(2, Value::String("hello".to_string()).into())
            .exit_scope(2)
            .add_load_instruction(4, RegisterValue::ScopeId(1, 2))
            .add_halt_instruction(4);
        let mut vm = builder.build();
        assert_eq!(vm.eval().unwrap(), Value::String("hello".to_string()))
    }

    #[test]
    fn function_scope() {
        let mut builder = VMBuilder::new();
        builder
            .enter_scope()
            .add_binary_instruction(BinaryOperation::Add, 1, 2, 3)
            .exit_scope(3)
            .add_load_instruction(1, RegisterValue::Value(1.into()))
            .add_load_instruction(2, RegisterValue::Value(2.into()))
            .add_call_instruction(1, 3)
            .add_load_instruction(1, RegisterValue::Register(3))
            .add_load_instruction(2, RegisterValue::Value(3.into()))
            .add_call_instruction(1, 3)
            .add_load_instruction(1, RegisterValue::Register(3))
            .add_load_instruction(2, RegisterValue::Value(4.into()))
            .add_call_instruction(1, 3)
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
                        Instruction::Load(
                            89,
                            RegisterValue::Value(
                                2.into(),
                            ),
                        ),
                        Instruction::LoadMutRegister(
                            "a",
                            89,
                        ),
                        Instruction::GetMutableVariable(
                            "a",
                            85,
                        ),
                        Instruction::CallSelf(
                            1,
                            85,
                            85,
                            true,
                        ),
                        Instruction::CallSelf(
                            1,
                            85,
                            85,
                            true,
                        ),
                        Instruction::CallSelf(
                            1,
                            85,
                            85,
                            true,
                        ),
                        Instruction::GetVariable(
                            "a",
                            90,
                        ),
                        Instruction::Halt(
                            90,
                        ),
                    ],
                    owned_registers: vec![],
                },
                Scope {
                    instructions: vec![
                        Instruction::GetSelf(
                            86,
                            true,
                        ),
                        Instruction::Load(
                            87,
                            RegisterValue::Value(
                                3.into(),
                            ),
                        ),
                        Instruction::BinaryAssign(
                            BinaryAssign {
                                op: BinaryOperation::Mul,
                                lhs: 86,
                                rhs: 87,
                            },
                        ),
                        Instruction::GetSelf(
                            88,
                            true,
                        ),
                        Instruction::Load(
                            85,
                            RegisterValue::Register(88),
                        ),
                        Instruction::Ret(
                            85,
                        ),
                    ],
                    owned_registers: vec![],
                },
            ],
            ..Default::default()
        };
        assert_eq!(vm.run(), 54.into(), "Run Failed {vm:#?}")
    }
}
