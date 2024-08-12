use std::collections::HashMap;
use logos::Logos;
use rigz_vm::{RigzType, VMBuilder, VMError, VM};
use crate::fsm::{FunctionDefinition, State, FSM};
use crate::token::TokenKind;

mod fsm;
mod token;

pub fn parse(input: &str) -> Result<VM, VMError> {
    let lexer = TokenKind::lexer(input);
    let builder = VMBuilder::new();
    let mut fsm = FSM {
        lexer,
        builder,
        current: State::Empty,
        function_declarations: HashMap::from([
            ("puts".into(), FunctionDefinition {
                arguments: vec![RigzType::Any], //TODO add union type for Any
                return_type: RigzType::None,
            })
        ]),
        prior_state: Vec::new(),
        next_register: 2,
        last_register: 0,
    };

    loop {
        let token = match fsm.next_token() {
            Ok(o) => match o {
                None => break,
                Some(t) => t,
            },
            Err(e) => return Err(fsm.lex_error(e)),
        };
        fsm.process(token)?;
    }
    fsm.handle_remaining()
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
            .push(Instruction::LoadLet("a".into(), Value::String("Hello World".into())));
        scope.instructions.push(Instruction::Halt(0));
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
            .push(Instruction::LoadLet("a".to_string(), Value::String("Hello".into())));
        scope
            .instructions
            .push(Instruction::LoadLet("b".to_string(), Value::String("Elliot".into())));
        scope
            .instructions
            .push(Instruction::GetVariable("a".to_string(), 2));
        scope
            .instructions
            .push(Instruction::Load(3, Value::String(" ".into())));
        scope.instructions.push(Instruction::Binary {
            op: BinaryOperation::Add,
            lhs: 2,
            rhs: 3,
            output: 4,
        });
        scope
            .instructions
            .push(Instruction::GetVariable("b".to_string(), 5));
        scope.instructions.push(Instruction::Binary {
            op: BinaryOperation::Add,
            lhs: 4,
            rhs: 5,
            output: 6,
        });
        scope.instructions.push(Instruction::Halt(6));
        assert_eq!(vec![scope], vm.scopes)
    }
}