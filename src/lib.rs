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
mod vm;
mod value_range;

use std::fmt::Debug;
use std::hash::Hash;

pub use builder::VMBuilder;
pub use call_frame::{CallFrame, Variable};
pub use instructions::{Binary, BinaryOperation, Clear, Instruction, Unary, UnaryOperation};
pub use module::Module;
pub use number::Number;
pub use objects::RigzType;
pub use scope::Scope;
pub use traits::{Logical, Reverse};
pub use value::Value;
pub use vm::VM;

pub type Register = usize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    use crate::{Module, RigzType, VMBuilder, VMError, VM};
    use std::str::FromStr;

    #[test]
    fn load_works() {
        let mut builder = VMBuilder::new();
        builder.add_load_instruction(4, Value::Number(Number::Int(42)).into());
        let mut vm = builder.build();
        vm.eval().unwrap();
        assert_eq!(
            vm.registers.get(&4).unwrap().clone(),
            Value::Number(Number::Int(42)).into()
        );
    }

    #[test]
    fn cast_works() {
        let mut builder = VMBuilder::new();
        builder
            .add_load_instruction(4, Value::Number(Number::Int(42)).into())
            .add_cast_instruction(4, RigzType::String, 7);
        let mut vm = builder.build();
        vm.eval().unwrap();
        assert_eq!(
            vm.registers.get(&7).unwrap().clone(),
            Value::String(42.to_string()).into()
        );
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
        assert_eq!(
            vm.registers.get(&82).unwrap().clone(),
            Value::Number(Number::Int(84)).into()
        );
    }

    #[test]
    fn copy_works() {
        let mut builder = VMBuilder::new();
        builder
            .add_load_instruction(4, Value::Number(Number::Int(42)).into())
            .add_copy_instruction(4, 37);
        let mut vm = builder.build();
        vm.eval().unwrap();
        assert_eq!(
            vm.registers.get(&37).unwrap().clone(),
            Value::Number(Number::Int(42)).into()
        );
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
            vm.registers.get(&4).unwrap().clone(),
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
            vm.registers.get(&4).unwrap().clone(),
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
            vm.registers.get(&3).unwrap().clone(),
            Value::String(String::from_str("abc").unwrap()).into()
        );
    }

    #[derive(Clone)]
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

        fn call_extension(
            &self,
            value: Value,
            function: &'vm str,
            args: Vec<Value>,
        ) -> Result<Value, VMError> {
            Err(VMError::InvalidModuleFunction(function.to_string()))
        }

        fn vm_extension(
            &self,
            vm: &mut VM<'vm>,
            function: &'vm str,
            args: Vec<Value>,
        ) -> Result<Value, VMError> {
            Err(VMError::InvalidModuleFunction(function.to_string()))
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
        assert_eq!(vm.registers.get(&3).unwrap().clone(), Value::None.into());
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
}
