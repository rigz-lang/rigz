use logos::Logos;
pub use rigz_vm::{RigzType, VM, Value, Number, Module, VMBuilder};
use crate::parser::Parser;
use crate::token::{LexingError};

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub arguments: Vec<RigzType>,
    pub return_type: RigzType,
}

mod token;
mod parser;

pub fn parse(input: &str) -> Result<VM, LexingError> {
    Parser::parse(input)
}

pub fn parse_with_modules<'vm>(input: &'vm str, modules: Vec<Module<'vm>>) -> Result<VM<'vm>, LexingError> {
    let mut builder = VMBuilder::new();
    for module in modules {
        builder.register_module(module);
    }
    Parser::parse_with_builder(input, builder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rigz_vm::{BinaryOperation, Instruction, Scope, UnaryOperation, Value};

    #[test]
    fn create_vm_works() {
        let a = "puts 'Hello World'";
        let vm = parse(a).unwrap();
        let mut scope = Scope::new();
        scope
            .instructions
            .push(Instruction::Load(2, Value::String("Hello World".into())));
        scope.instructions.push(Instruction::Unary {
            op: UnaryOperation::Print,
            from: 2,
            output: 0,
        });
        scope.instructions.push(Instruction::Halt(0));
        assert_eq!(vec![scope], vm.scopes)
    }

    #[test]
    fn parse_string() {
        let a = "'Hello World'";
        let vm = parse(a).unwrap();
        let mut scope = Scope::new();
        scope
            .instructions
            .push(Instruction::Load(2, Value::String("Hello World".into())));
        scope.instructions.push(Instruction::Halt(2));
        assert_eq!(vec![scope], vm.scopes)
    }

    #[test]
    fn parse_assignment() {
        let a = "a = 'Hello World'";
        let vm = parse(a).unwrap();
        let mut scope = Scope::new();
        scope
            .instructions
            .push(Instruction::Load(2, Value::String("Hello World".into())));
        scope
            .instructions
            .push(Instruction::LoadLetRegister("a".into(), 2));
        scope.instructions.push(Instruction::Halt(2));
        assert_eq!(vec![scope], vm.scopes)
    }

    #[test]
    fn parse_simple() {
        let a = r#"
            a = "Hello"
            b = "Elliot"
            a + " " + b
        "#;
        let vm = parse(a).unwrap();
        let mut scope = Scope::new();
        scope
            .instructions
            .push(Instruction::Load(2, Value::String("Hello".into())));
        scope
            .instructions
            .push(Instruction::LoadLetRegister("a".into(), 2));
        scope
            .instructions
            .push(Instruction::Load(3, Value::String("Elliot".into())));
        scope
            .instructions
            .push(Instruction::LoadLetRegister("b".into(), 3));
        scope
            .instructions
            .push(Instruction::GetVariable("a".to_string(), 4));
        scope
            .instructions
            .push(Instruction::Load(5, Value::String(" ".into())));
        scope
            .instructions
            .push(Instruction::GetVariable("b".to_string(), 6));
        scope.instructions.push(Instruction::Binary {
            op: BinaryOperation::Add,
            lhs: 5,
            rhs: 6,
            output: 7,
        });
        scope.instructions.push(Instruction::Binary {
            op: BinaryOperation::Add,
            lhs: 4,
            rhs: 7,
            output: 8,
        });
        scope.instructions.push(Instruction::Halt(8));
        assert_eq!(vm.scopes[0], scope)
    }
}