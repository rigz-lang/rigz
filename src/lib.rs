use std::collections::HashMap;
use std::fmt::Display;
use std::str::ParseBoolError;
use logos::{Lexer, Logos, Span};
use rigz_vm::{BinaryOperation, Number, Register, UnaryOperation, VMBuilder, VMError, Value, VM};
use crate::TokenKind::BinOp;

#[derive(Debug, PartialEq, Clone, Default)]
enum LexingError {
    NumberParseError,
    #[default]
    Other,
    BoolParseError,
}

impl From<std::num::ParseIntError> for LexingError {
    fn from(_: std::num::ParseIntError) -> Self {
        LexingError::NumberParseError
    }
}

impl From<std::num::ParseFloatError> for LexingError {
    fn from(_: std::num::ParseFloatError) -> Self {
        LexingError::NumberParseError
    }
}

impl From<ParseBoolError> for LexingError {
    fn from(_: ParseBoolError) -> Self {
        LexingError::BoolParseError
    }
}

/*
Value = {None, Bool, Int, Float, String, List, Map }
- [Identifier, Eq, Value, !bin_op]
- [Value, !bin_op]
- [FunctionIdentifier, Value, !bin_op]
- [Value, bin_op, Value, !bin_op]
 */

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(error = LexingError)]
enum TokenKind {
    #[regex(r"[ \n\t\f]+", logos::skip)]
    Ignored,
    #[regex("-?[0-9]+", |lex| lex.slice().parse())]
    Integer(i64),
    #[regex("-?[0-9]+\\.[0-9]+", |lex| lex.slice().parse())]
    Float(f64),
    #[regex("true|false", |lex| lex.slice().parse())]
    Bool(bool),
    #[regex("('[^'\n\r]+')|(\"[^\"\n\r]+\")|(`[^`\n\r]+`)", |lex| { let s = lex.slice(); s[1..s.len()-1].to_string() })]
    StrLiteral(String),
    // TODO double & backticks string support interpolation
    #[token("none")]
    None,
    #[token("=")]
    Eq,
    #[token("==", |_| BinaryOperation::Eq)]
    #[token("!=", |_| BinaryOperation::Neq)]
    #[token("<", |_| BinaryOperation::Lt)]
    #[token(">>", |_| BinaryOperation::Shr)]
    #[token("<<", |_| BinaryOperation::Shl)]
    #[token(">", |_| BinaryOperation::Gt)]
    #[token("<=", |_| BinaryOperation::Lte)]
    #[token(">=", |_| BinaryOperation::Gte)]
    #[token("+", |_| BinaryOperation::Add)]
    #[token("-", |_| BinaryOperation::Sub)]
    #[token("*", |_| BinaryOperation::Mul)]
    #[token("/", |_| BinaryOperation::Div)]
    #[token("%", |_| BinaryOperation::Rem)]
    #[token("&&", |_| BinaryOperation::And)]
    #[token("||", |_| BinaryOperation::Or)]
    #[token("&", |_| BinaryOperation::BitAnd)]
    #[token("|", |_| BinaryOperation::BitOr)]
    #[token("^", |_| BinaryOperation::Xor)]
    BinOp(BinaryOperation),
    #[token("!", |_| UnaryOperation::Not)]
    #[token("-", |_| UnaryOperation::Neg, priority=3)]
    UnaryOp(UnaryOperation),
    #[token(".")]
    Period,
    #[token(",")]
    Comma,
    #[token("fn")]
    FunctionDef,
    #[regex("[A-Za-z_]+", |lex| lex.slice().to_string())]
    Identifier(String),
    #[regex("[A-Za-z_$]+", |lex| lex.slice().to_string(), priority=3)]
    FunctionIdentifier(String),
    #[token("(")]
    Lparen,
    #[token(")")]
    Rparen,
    #[token("{")]
    Lcurly,
    #[token("}")]
    Rcurly,
    #[token("[")]
    Lbracket,
    #[token("]")]
    Rbracket,
}

impl TokenKind {
    pub fn is_value(&self) -> bool {
        match self {
            TokenKind::None | TokenKind::Bool(_) | TokenKind::Integer(_) | TokenKind::Float(_) | TokenKind::StrLiteral(_) => true,
             _ => false
        }
    }

    pub fn is_identifier(&self) -> bool {
        if let TokenKind::Identifier(_) = self {
            true
        } else {
            false
        }
    }
}

pub struct Token<'vm> {
    kind: TokenKind,
    span: Span,
    slice: &'vm str
}

impl <'vm> Token<'vm> {
    #[inline]
    pub fn parse_error(&self, message: String) -> VMError {
        VMError::ParseError(message, self.span.start, self.span.end)
    }
}

impl <'vm> Default for Token<'vm> {
    fn default() -> Self {
        Self {
            kind: TokenKind::Ignored,
            slice: "",
            span: Span::default(),
        }
    }
}

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
            function_declarations: HashMap::from([("puts".to_string(), 0)])
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
                                        Token { kind: TokenKind::Identifier(s.to_string()), span, slice }
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
            Some(t) => Some(self.next(t)?)
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
                                fsm.builder.add_load_instruction(reg, Value::Number(Number::Int(i)));
                            }
                        }
                        TokenKind::Float(f) => {
                            if is_value {
                                let reg = fsm.next_register();
                                fsm.builder.add_load_instruction(reg, Value::Number(Number::Float(f)));
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
                                          TokenKind::FunctionIdentifier(f) => {
                                              match f.as_str() {
                                                  "puts" => {
                                                      let last = fsm.last_register;
                                                      fsm.set_last(0);
                                                      fsm.builder.add_print_instruction(last, fsm.last_register);
                                                  }
                                                  f => {
                                                      return Err(token.parse_error(format!("Unsupported Function: {}", f)))
                                                  }
                                              }
                                          }
                                          k => return Err(token.parse_error(format!("Unsupported operation: {:?}", k)))
                                      }
                                    },
                                    0 => unreachable!(),
                                    _ => todo!()
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
                        k => return Err(fsm.current.parse_error(format!("Unsupported Token as last element {:?}", k)))
                    }
                }
                break
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
                    TokenKind::Identifier(_) => {
                        match fsm.needs_token.last() {
                            None => {
                                println!("Empty tokens")
                            }
                            Some(s) => {
                                match &s.kind {
                                    _ => {}
                                }
                            }
                        }
                    }
                    TokenKind::FunctionIdentifier(_) => {}
                    TokenKind::Rparen => {}
                    TokenKind::Rcurly => {}
                    TokenKind::Rbracket => {}
                    _ => {}
                }
            },
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
    use rigz_vm::{Instruction, Scope, UnaryOperation, Value};
    use super::*;

    #[test]
    fn tokenize_works() {
        let raw = r#"
            a = 1
            b = 2
            a + b
        "#;
        // current (Identifier, a, 0..1)
        // current (Eq, =, 2..3), stack: {identifier}
        // current (Number, 1, 4..5), stack: {Eq, identifier}
        // vm.

        let lexer = TokenKind::lexer(raw);
        let actual: Vec<TokenKind> = lexer.map(|t| t.unwrap()).collect();
        assert_eq!(actual, vec![
            TokenKind::FunctionIdentifier("a".to_string()),
            TokenKind::Eq,
            TokenKind::Integer(1),
            TokenKind::FunctionIdentifier("b".to_string()),
            TokenKind::Eq,
            TokenKind::Integer(2),
            TokenKind::FunctionIdentifier("a".to_string()),
            TokenKind::BinOp(BinaryOperation::Add),
            TokenKind::FunctionIdentifier("b".to_string()),
        ])
    }

    #[test]
    fn create_vm_works() {
        let a = "puts 'Hello World'";
        let vm = parse(a).unwrap();
        let mut scope = Scope::new();
        scope.instructions.push(Instruction::Load(2, Value::String("Hello World".into())));
        scope.instructions.push(Instruction::Unary {
            op: UnaryOperation::Print,
            from: 2,
            output: 0
        });
        scope.instructions.push(Instruction::Halt(0));
        assert_eq!(vec![
            scope
        ], vm.scopes)
    }

    #[test]
    fn parse_string() {
        let a = "'Hello World'";
        let vm = parse(a).unwrap();
        let mut scope = Scope::new();
        scope.instructions.push(Instruction::Load(2, Value::String("Hello World".into())));
        scope.instructions.push(Instruction::Halt(2));
        assert_eq!(vec![
            scope
        ], vm.scopes)
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
        scope.instructions.push(Instruction::Load(2, Value::String("Hello".into())));
        scope.instructions.push(Instruction::LoadMutRegister("a".to_string(), 2));
        scope.instructions.push(Instruction::Load(3, Value::String("Elliot".into())));
        scope.instructions.push(Instruction::LoadMutRegister("b".to_string(), 3));
        scope.instructions.push(Instruction::GetVariable("a".to_string(), 4));
        scope.instructions.push(Instruction::Load(5, Value::String(" ".into())));
        scope.instructions.push(Instruction::Binary {
            op: BinaryOperation::Add,
            lhs: 4,
            rhs: 5,
            output: 6
        });
        scope.instructions.push(Instruction::GetVariable("b".to_string(), 7));
        scope.instructions.push(Instruction::Binary {
            op: BinaryOperation::Add,
            lhs: 6,
            rhs: 7,
            output: 8
        });
        scope.instructions.push(Instruction::Halt(8));
        assert_eq!(vec![
            scope
        ], vm.scopes)
    }
}
