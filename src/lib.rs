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

pub(crate) use objects::{BOOL, ERROR, LIST, MAP, NONE, NUMBER, STRING};
use std::fmt::Debug;
use std::hash::Hash;

pub use builder::VMBuilder;
pub use call_frame::{CallFrame, Variable};
pub use instructions::{Binary, BinaryOperation, Instruction, Unary, UnaryOperation};
pub use lifecycle::{Message, Lifecycle};
pub use module::{Function, ExtensionFunction, Module};
pub use number::Number;
pub use objects::{RigzObject, RigzObjectDefinition, RigzType};
pub use scope::Scope;
pub use traits::{Logical, Reverse};
pub use value::Value;
pub use vm::VM;

pub type Register = usize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VMError {
    RuntimeError(String),
    FrameError(String),
    ScopeError(String),
    InvalidPC(String),
    EmptyRegister(String),
    ConversionError(String),
    ScopeDoesNotExist(String),
    UnsupportedOperation(String),
    ParseError(String, usize, usize),
    VariableDoesNotExist(String),
    InvalidModule(String),
    InvalidModuleFunction(String),
    LifecycleError(String),
}

impl<'vm> VMError {
    pub fn to_value(self) -> Value<'vm> {
        Value::Error(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::number::Number;
    use crate::value::Value;
    use crate::{Module, RigzType, VMBuilder};
    use indexmap::IndexMap;
    use std::str::FromStr;

    #[test]
    fn value_eq() {
        assert_eq!(Value::None, Value::None);
        assert_eq!(Value::None, Value::Bool(false));
        assert_eq!(Value::None, Value::Number(Number::Int(0)));
        assert_eq!(Value::None, Value::Number(Number::Float(0.0)));
        assert_eq!(Value::None, Value::Number(Number::UInt(0)));
        assert_eq!(Value::None, Value::String(String::new()));
        assert_eq!(Value::Bool(false), Value::String(String::new()));
        assert_eq!(Value::Number(Number::UInt(0)), Value::String(String::new()));
    }

    #[test]
    fn load_works() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(4, Value::Number(Number::Int(42)))
            .build();
        vm.run().unwrap();
        assert_eq!(
            vm.registers.get(&4).unwrap().clone(),
            Value::Number(Number::Int(42))
        );
    }

    #[test]
    fn cast_works() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(4, Value::Number(Number::Int(42)))
            .add_cast_instruction(4, RigzType::String, 7)
            .build();
        vm.run().unwrap();
        assert_eq!(
            vm.registers.get(&7).unwrap().clone(),
            Value::String(42.to_string())
        );
    }

    #[test]
    fn add_works() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(4, Value::Number(Number::Int(42)))
            .add_copy_instruction(4, 37)
            .add_add_instruction(4, 37, 82)
            .build();
        vm.run().unwrap();
        assert_eq!(
            vm.registers.get(&82).unwrap().clone(),
            Value::Number(Number::Int(84))
        );
    }

    #[test]
    fn copy_works() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(4, Value::Number(Number::Int(42)))
            .add_copy_instruction(4, 37)
            .build();
        vm.run().unwrap();
        assert_eq!(
            vm.registers.get(&37).unwrap().clone(),
            Value::Number(Number::Int(42))
        );
    }

    #[test]
    fn shr_works_str_number() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(2, Value::String(String::from_str("abc").unwrap()))
            .add_load_instruction(3, Value::Number(Number::Int(1)))
            .add_shr_instruction(2, 3, 4)
            .build();
        vm.run().unwrap();
        assert_eq!(
            vm.registers.get(&4).unwrap().clone(),
            Value::String(String::from_str("ab").unwrap())
        );
    }

    #[test]
    fn shl_works_str_number() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(2, Value::String(String::from_str("abc").unwrap()))
            .add_load_instruction(3, Value::Number(Number::Int(1)))
            .add_shl_instruction(2, 3, 4)
            .build();
        vm.run().unwrap();
        assert_eq!(
            vm.registers.get(&4).unwrap().clone(),
            Value::String(String::from_str("bc").unwrap())
        );
    }

    #[test]
    fn call_works() {
        let mut builder = VMBuilder::new();
        let mut vm = builder
            .add_load_instruction(2, Value::String(String::from_str("abc").unwrap()))
            .enter_scope()
            .add_copy_instruction(2, 3)
            .exit_scope(3)
            .add_call_instruction(1, 3)
            .build();
        vm.run().unwrap();
        assert_eq!(
            vm.registers.get(&3).unwrap().clone(),
            Value::String(String::from_str("abc").unwrap())
        );
    }

    #[test]
    fn module_works<'vm>() {
        let mut builder = VMBuilder::new();
        fn hello(args: Vec<Value>) -> Value {
            println!("{}", Value::List(args));
            Value::None
        }
        let mut functions: IndexMap<&'vm str, &dyn Fn(Vec<Value<'vm>>) -> Value<'vm>> =
            IndexMap::new();
        functions.insert("hello", &hello);

        let module = Module {
            name: "test",
            functions,
            extension_functions: Default::default(),
        };
        let mut vm = builder
            .register_module(module)
            .add_load_instruction(2, Value::String(String::from_str("abc").unwrap()))
            .add_call_module_instruction("test", "hello", vec![2], 3)
            .build();
        vm.run().unwrap();
        assert_eq!(vm.registers.get(&3).unwrap().clone(), Value::None);
    }

    #[test]
    fn assignment_scopes_work() {
        pretty_env_logger::init();
        let mut builder = VMBuilder::new();
        // a = 1 + 2; a + 2
        builder
            .enter_scope()
            .add_load_instruction(2, Value::Number(Number::Int(1)))
            .add_load_instruction(3, Value::Number(Number::Int(2)))
            .add_add_instruction(2, 3, 4)
            .exit_scope(4)
            .add_load_instruction(5, Value::ScopeId(1, 4))
            .add_load_let_instruction("a", 5)
            .add_get_variable_instruction("a", 6)
            .add_load_instruction(7, Value::Number(Number::Int(2)))
            .add_add_instruction(6, 7, 8)
            .add_halt_instruction(8);

        let mut vm = builder.build();
        let v = vm.run().unwrap();
        assert_eq!(v, Value::Number(Number::Int(5)))
    }

    #[test]
    fn simple_scope() {
        let mut builder = VMBuilder::new();
        builder
            .enter_scope()
            .add_load_instruction(2, Value::String("hello".to_string()))
            .exit_scope(2)
            .add_load_instruction(4, Value::ScopeId(1, 2))
            .add_halt_instruction(4);
        let mut vm = builder.build();
        assert_eq!(vm.run().unwrap(), Value::String("hello".to_string()))
    }
}
