mod fsm;
mod token;

use logos::{Lexer, Logos, Span};
use rigz_vm::{BinaryOperation, Number, Register, UnaryOperation, VMBuilder, VMError, Value, VM};
use std::collections::HashMap;
use std::fmt::Display;

/*
Value = {None, Bool, Int, Float, String, List, Map }
- [Identifier, Eq, Value, !bin_op]
- [Value, !bin_op]
- [FunctionIdentifier, Value, !bin_op]
- [Value, bin_op, Value, !bin_op]
 */
pub struct StateMachine<'vm> {
    builder: VMBuilder<'vm>,
    lexer: Lexer<'vm, TokenKind>,
    current: Token<'vm>,
    needs_token: Vec<Token<'vm>>,
    next_register: Register,
    last_register: Register,
    function_declarations: HashMap<String, usize>,
}

impl<'vm> StateMachine<'vm> {
    fn lex_next(lexer: &mut Lexer<'vm, TokenKind>) -> Option<Token<'vm>> {
        match lexer.next() {
            None => None,
            Some(t) => {
                let span = lexer.span();
                let slice = lexer.slice();
                Some(Token {
                    kind: t.expect("Failed to parse token"),
                    span,
                    slice,
                })
            }
        }
    }

    pub fn new(input: &'vm str) -> StateMachine<'vm> {
        let mut lexer = TokenKind::lexer(input);
        // TODO I don't like this expect
        let current = Self::lex_next(&mut lexer).expect("Attempted to lex no tokens");
        Self {
            lexer,
            builder: VMBuilder::new(),
            current,
            needs_token: vec![],
            next_register: 2,
            last_register: 0,
            function_declarations: HashMap::from([("puts".to_string(), 0)]),
        }
    }

    pub fn next(&mut self, token: Token<'vm>) -> Result<(), VMError> {
        match self.needs_token.last() {
            Some(t) => {
                match (&t.kind, &self.current.kind, &token.kind) {
                    _ => {
                        let prev = std::mem::take(&mut self.current);
                        let token = match token {
                            Token { kind, span, slice } => {
                                if let TokenKind::FunctionIdentifier(s) = &kind {
                                    if self.function_declarations.contains_key(s) {
                                        Token { kind, span, slice }
                                    } else {
                                        // TODO this isn't very efficient
                                        Token {
                                            kind: TokenKind::Identifier(s.to_string()),
                                            span,
                                            slice,
                                        }
                                    }
                                } else {
                                    Token { kind, span, slice }
                                }
                            }
                        };
                        self.current = token;
                        self.needs_token.push(prev);
                    }
                }
            }
            None => {
                let prev = std::mem::take(&mut self.current);
                self.current = token;
                self.needs_token.push(prev);
            }
        }
        Ok(())
    }

    pub fn set_last(&mut self, last: Register) {
        self.last_register = last;
    }

    pub fn next_register(&mut self) -> Register {
        let next = self.next_register;
        self.set_last(next);
        match self.next_register.checked_add(1) {
            None => panic!("Registers have exceeded u64::MAX"),
            Some(r) => {
                self.next_register = r;
            }
        };
        next
    }

    pub fn next_token(&mut self) -> Result<Option<()>, VMError> {
        let token = Self::lex_next(&mut self.lexer);
        let o = match token {
            None => None,
            Some(t) => Some(self.next(t)?),
        };
        Ok(o)
    }
}

/**
identifer; current = identifier
eq; current = eq, needs_token [identifier]
string; current = string, needs_token(identifier, eq)
identifier; current = identifier, needs_token [], vm_builder (n instructions)
*/

pub fn parse<'vm>(input: &'vm str) -> Result<VM<'vm>, VMError> {
    let mut fsm = StateMachine::new(input);
    loop {
        match fsm.next_token()? {
            None => {
                // TODO remove clone here
                let kind = fsm.current.kind.clone();
                if kind != TokenKind::Ignored {
                    let is_value = fsm.needs_token.is_empty();
                    match kind {
                        TokenKind::Integer(i) => {
                            if is_value {
                                let reg = fsm.next_register();
                                fsm.builder
                                    .add_load_instruction(reg, Value::Number(Number::Int(i)));
                            }
                        }
                        TokenKind::Float(f) => {
                            if is_value {
                                let reg = fsm.next_register();
                                fsm.builder
                                    .add_load_instruction(reg, Value::Number(Number::Float(f)));
                            }
                        }
                        TokenKind::Bool(b) => {
                            if is_value {
                                let reg = fsm.next_register();
                                fsm.builder.add_load_instruction(reg, Value::Bool(b));
                            }
                        }
                        TokenKind::StrLiteral(s) => {
                            let reg = fsm.next_register();
                            fsm.builder.add_load_instruction(reg, Value::String(s));
                            if !is_value {
                                match fsm.needs_token.len() {
                                    1 => {
                                        let token = fsm.needs_token.pop().unwrap();
                                        match &token.kind {
                                            TokenKind::FunctionIdentifier(f) => match f.as_str() {
                                                "puts" => {
                                                    let last = fsm.last_register;
                                                    fsm.set_last(0);
                                                    fsm.builder.add_print_instruction(
                                                        last,
                                                        fsm.last_register,
                                                    );
                                                }
                                                f => {
                                                    return Err(token.parse_error(format!(
                                                        "Unsupported Function: {}",
                                                        f
                                                    )))
                                                }
                                            },
                                            k => {
                                                return Err(token.parse_error(format!(
                                                    "Unsupported operation: {:?}",
                                                    k
                                                )))
                                            }
                                        }
                                    }
                                    0 => unreachable!(),
                                    _ => todo!(),
                                }
                            }
                        }
                        TokenKind::None => {
                            if is_value {
                                let reg = fsm.next_register();
                                fsm.builder.add_load_instruction(reg, Value::None);
                            }
                        }
                        TokenKind::Identifier(s) => {
                            if is_value {
                                let reg = fsm.next_register();
                                fsm.builder.add_get_variable_instruction(s, reg);
                            }
                        }
                        TokenKind::FunctionIdentifier(s) => {
                            todo!()
                        }
                        k => {
                            return Err(fsm
                                .current
                                .parse_error(format!("Unsupported Token as last element {:?}", k)))
                        }
                    }
                }
                break;
            }
            Some(_) => {
                let kind = &fsm.current.kind;
                println!("{:?}", kind);
                match kind {
                    TokenKind::Integer(_) => {}
                    TokenKind::Float(_) => {}
                    TokenKind::Bool(_) => {}
                    TokenKind::StrLiteral(_) => {}
                    TokenKind::None => {}
                    TokenKind::BinOp(_) => {}
                    TokenKind::UnaryOp(_) => {}
                    TokenKind::Period => {}
                    TokenKind::Comma => {}
                    TokenKind::FunctionDef => {}
                    TokenKind::Identifier(_) => match fsm.needs_token.last() {
                        None => {
                            println!("Empty tokens")
                        }
                        Some(s) => match &s.kind {
                            _ => {}
                        },
                    },
                    TokenKind::FunctionIdentifier(_) => {}
                    TokenKind::Rparen => {}
                    TokenKind::Rcurly => {}
                    TokenKind::Rbracket => {}
                    _ => {}
                }
            }
        }
    }

    if fsm.builder.sp == 0 {
        fsm.builder.add_halt_instruction(fsm.last_register);
    }

    let vm = fsm.builder.build();
    Ok(vm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rigz_vm::{Instruction, Scope, UnaryOperation, Value};

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
            .push(Instruction::LoadMutRegister("a".to_string(), 2));
        scope
            .instructions
            .push(Instruction::Load(3, Value::String("Elliot".into())));
        scope
            .instructions
            .push(Instruction::LoadMutRegister("b".to_string(), 3));
        scope
            .instructions
            .push(Instruction::GetVariable("a".to_string(), 4));
        scope
            .instructions
            .push(Instruction::Load(5, Value::String(" ".into())));
        scope.instructions.push(Instruction::Binary {
            op: BinaryOperation::Add,
            lhs: 4,
            rhs: 5,
            output: 6,
        });
        scope
            .instructions
            .push(Instruction::GetVariable("b".to_string(), 7));
        scope.instructions.push(Instruction::Binary {
            op: BinaryOperation::Add,
            lhs: 6,
            rhs: 7,
            output: 8,
        });
        scope.instructions.push(Instruction::Halt(8));
        assert_eq!(vec![scope], vm.scopes)
    }
}
