use crate::token::{LexingError, Token, TokenKind};
use indexmap::IndexMap;
use logos::{Lexer, Logos};
use rigz_vm::{BinaryOperation, Number, Register, RigzType, Scope, UnaryOperation, VMBuilder, VMError, Value, VM};
use std::collections::HashMap;
use std::fmt::format;

// ```
// enter_scope
// load 2, 2
// bin_op add, 1, 2, 3
// ret 3
// exit
//
// Empty
// Assignment (since a is not a defined function)
// Assignment (a, =)
// Assignment(a, 1)
//
// current = Expression(BinOp(+), 1) prior(Assignment(a, do)))
// current = Expression(BinOp(+), 2) prior(Primitive(1), Assignment(a, do))
// current = Assignment(b) prior = [], builder needs to be applied
// current = Assignment(b, =)
// current = Assignment(b, 2)
//
//
// a = load_variable(a, scope(1))
// load 4, 2
// b = load_variable(b, 4)
//
// a = 1 + 2
// b = 2
// a + b
// ```

#[derive(Debug, Clone, Default)]
pub enum State<'vm> {
    #[default]
    Empty,
    One(Token<'vm>),
    Primitive(Primitive),
    FunctionCall(String, Token<'vm>),
    // Def(usize, Token<'vm>),
    // Fn(String, Vec<RigzType>, RigzType, Token<'vm>),
    BinOp(BinaryOperation, Token<'vm>),
    // UnaryOp(UnaryOperation, Token<'vm>),
    // Expression(Token<'vm>),
    // Map(IndexMap<Value<'vm>, Value<'vm>>, Token<'vm>),
    // List(Vec<Value<'vm>>, Token<'vm>),
    // // Import,
    // // Export
    // Scope(usize, Token<'vm>),
    Invalid(Token<'vm>),
    //
    Assignment(String, Token<'vm>),
    MutAssignment(String, Token<'vm>),
}

impl <'vm> State<'vm> {
    pub fn token(&self) -> Option<Token<'vm>> {
        let t = match self {
            State::One(t) => t,
            State::FunctionCall(_, t) => t,
            State::Invalid(t) => t,
            State::Assignment(_, t) => t,
            _ => return None,
        }.clone();
        Some(t)
    }
}

#[derive(Debug, Clone)]
pub enum Primitive {
    None,
    Bool(bool),
    Number(Number),
    String(String),
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub arguments: Vec<RigzType>,
    pub return_type: RigzType,
}

#[derive(Debug)]
pub struct FSM<'vm> {
    pub(crate) lexer: Lexer<'vm, TokenKind>,
    pub(crate) builder: VMBuilder<'vm>,
    pub(crate) current: State<'vm>,
    pub(crate) prior_state: Vec<State<'vm>>,
    pub(crate) function_declarations: HashMap<String, FunctionDefinition>,
    pub(crate) next_register: Register,
    pub(crate) last_register: Register,
}

impl<'vm> FSM<'vm> {
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

    pub fn process(&mut self, token: Token<'vm>) -> Result<(), VMError> {
        match (&self.current, token) {
            (State::Empty, token) => match &token.kind {
                TokenKind::Ignored => {}
                TokenKind::Integer(i) => self.transition(State::Primitive(Primitive::Number(Number::Int(*i))))?,
                TokenKind::Float(f) => self.transition(State::Primitive(Primitive::Number(Number::Float(*f))))?,
                TokenKind::Bool(b) => self.transition(State::Primitive(Primitive::Bool(*b)))?,
                TokenKind::StrLiteral(s) => self.transition(State::Primitive(Primitive::String(s.clone())))?,
                TokenKind::None => self.transition(State::Primitive(Primitive::None))?,
                // TokenKind::FunctionDef => {
                //     self.builder.enter_scope();
                //     self.transition(State::Def(self.builder.sp, token))?;
                // }
                TokenKind::Identifier(i) => self.transition(State::Assignment(i.clone(), token))?,
                TokenKind::FunctionIdentifier(i) => {
                    if !self.function_declarations.contains_key(i) {
                        let Token { span, slice, .. } = token;
                        self.transition(State::Assignment(i.clone(), Token {
                            kind: TokenKind::Identifier(i.to_string()),
                            span,
                            slice,
                        }))?;
                    } else {
                        self.transition(State::FunctionCall(i.clone(), token))?;
                    }
                }
                // TokenKind::Lparen => self.transition(State::Expression(token))?,
                // TokenKind::Lcurly => self.transition(State::Map(IndexMap::new(), token))?,
                // TokenKind::Lbracket => self.transition(State::List(Vec::new(), token))?,
                TokenKind::Newline => self.transition(State::Empty)?,
                _ => self.transition(State::Invalid(token))?,
            },
            (State::Primitive(p), token) => {
                match &token.kind {
                    TokenKind::Ignored => {}
                    TokenKind::Assign | TokenKind::None | TokenKind::Bool(_) | TokenKind::Float(_) | TokenKind::Integer(_) | TokenKind::StrLiteral(_) =>
                        self.transition(State::Invalid(token))?,
                    //TokenKind::BinOp(o) => self.transition(State::BinOp(o.clone(), token))?,
                    _ => self.transition(State::Invalid(token))?,
                }
            }
            (State::Invalid(_), _) => {}
            (State::FunctionCall(name, last), token) => {
                match (&last.kind, &token.kind) {
                    (TokenKind::FunctionIdentifier(_), TokenKind::StrLiteral(_)) => {
                        self.transition(State::FunctionCall(name.clone(), token))?;
                    }
                    (TokenKind::FunctionIdentifier(_), TokenKind::Comma) => {

                    }
                    _ => todo!()
                }
            }
            (State::Assignment(n, last), token) => {
                if let TokenKind::Newline = token.kind {
                    self.transition(State::Empty)?;
                    return Ok(())
                }

                match (&last.kind, &token.kind) {
                    (TokenKind::Identifier(i), TokenKind::Assign) => self.transition(State::Assignment(i.to_string(), token))?,
                    (TokenKind::Identifier(i), TokenKind::BinOp(b)) => self.transition(State::BinOp(b.clone(), token))?,
                    (TokenKind::Assign, _) => {
                        self.transition(State::Assignment(n.to_string(), token))?
                    },
                    (a, b) => return Err(token.parse_error(format!("Invalid Combination of {:?} and {:?}", *a, *b)))
                }
            },
            (State::MutAssignment(n, last), token) => {
                if let TokenKind::Newline = token.kind {
                    self.transition(State::Empty)?;
                    return Ok(())
                }
            },
            (State::BinOp(b, last), token) => {

            }
            (s, next) => return Err(next.parse_error(format!("Unexpected State {:?} & Token {:?}", s, next)))
        }
        Ok(())
    }

    pub fn handle_remaining(&mut self) -> Result<VM<'vm>, VMError> {
        let current = std::mem::take(&mut self.current);
        self.process_state(current)?;
        loop {
            match self.prior_state.pop() {
                None => break,
                Some(s) => self.process_state(s)?,
            }
        }

        if self.builder.sp == 0 {
            let last = self.last_register;
            self.builder.add_halt_instruction(last);
        }
        Ok(self.builder.build())
    }

    pub fn transition(&mut self, state: State<'vm>) -> Result<(), VMError> {
        if let State::Invalid(t) = state {
            return Err(t.parse_error(format!("Unexpected Token {:?}", t)))
        }

        let last = std::mem::take(&mut self.current);
        match (last, &state) {
            (State::Empty, other) => {
                println!("other: {:?}", other)
            }
            (State::One(a), State::Assignment(_, _)) => {
                if !a.is_identifier() {
                   return Err(a.parse_error(format!("Unexpected Assignment {:?}", a)))
                }
            }
            (State::Assignment(name, token), State::Empty) => {
                if self.prior_state.is_empty() {
                    self.process_state(State::Assignment(name, token))?;
                } else {
                    loop {
                        match self.prior_state.pop() {
                            None => break,
                            Some(s) => self.process_state(s)?,
                        }
                    }
                    todo!()
                }
            }
            (State::FunctionCall(_, _), State::FunctionCall(_, _)) => {}
            (State::Assignment(_, _), State::Assignment(_, _)) => {}
            (State::Assignment(o, token), State::BinOp(_, _)) => {

                self.prior_state.push(State::Assignment(o, token));
            }
            (last, next) => return Err(VMError::ParseError(format!("Unimplemented transition {:?} {:?}", last, next), 0, usize::MAX))
        }
        self.current = state;
        Ok(())
    }

    pub fn process_state(&mut self, state: State<'vm>) -> Result<(), VMError> {
        match state {
            State::Empty => {}
            State::One(t) => {
                if t.is_value() {
                    let reg = self.next_register();
                    let v = t.to_value().unwrap();
                    self.builder.add_load_instruction(reg, v);
                }

                match &t.kind {
                    TokenKind::Ignored => {}
                    TokenKind::Comma => {}
                    TokenKind::Identifier(_) => {}
                    TokenKind::FunctionIdentifier(_) => {}
                    TokenKind::Rparen => {}
                    TokenKind::Rcurly => {}
                    TokenKind::Rbracket => {}
                    k => return Err(t.parse_error(format!("Invalid Remaining token {:?}", k))),
                }
            }
            State::Primitive(p) => {
                let reg = self.next_register();
                let value = match p {
                    Primitive::None => Value::None,
                    Primitive::Bool(b) => Value::Bool(b),
                    Primitive::Number(n) => Value::Number(n),
                    Primitive::String(s) => Value::String(s.clone()),
                };

                self.builder.add_load_instruction(reg, value);
            }
            State::Invalid(t) => return Err(t.parse_error("Invalid State".into())),
            State::FunctionCall(fc, token) => {
                if token.is_value() {
                    let reg = self.next_register();
                    self.builder.add_load_instruction(reg, token.to_value().unwrap());

                    match fc.as_str() {
                        "puts" => {
                            self.set_last(0);
                            self.builder.add_print_instruction(reg, 0);
                        }
                        _ => todo!()
                    }
                } else {
                    todo!()
                }
            }
            State::BinOp(_, _) => {

            }
            State::Assignment(name, token) => {
                if token.is_value() {
                    self.builder.add_load_let_instruction(name, token.to_value().unwrap());
                } else {
                    if let TokenKind::Identifier(s) = token.kind {
                        let next = self.next_register();
                        self.builder.add_get_variable_instruction(s, next);
                        self.builder.add_load_let_reg_instruction(name, next);
                    } else {
                        todo!()
                    }
                }
            }
            State::MutAssignment(name, token) => {
                if token.is_value() {
                    self.builder.add_load_mut_instruction(name, token.to_value().unwrap());
                } else {
                    if let TokenKind::Identifier(s) = token.kind {
                        let next = self.next_register();
                        self.builder.add_get_variable_instruction(s, next);
                        self.builder.add_load_mut_reg_instruction(name, next);
                    } else {
                        todo!()
                    }
                }
            }
        }
        Ok(())
    }

    pub fn lex_error(&self, lexing_error: LexingError) -> VMError {
        VMError::ParseError(format!("Failed to read input: {:?}", lexing_error), 0, 0)
    }

    pub fn next_token(&mut self) -> Result<Option<Token<'vm>>, LexingError> {
        let token = match self.lexer.next() {
            None => None,
            Some(t) => {
                let kind = t?;
                let slice = self.lexer.slice();
                let span = self.lexer.span();
                Some(Token { kind, slice, span })
            }
        };
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_works() {
        let raw = r#"
            a = 1
            b = 2
            a + b
        "#;

        let lexer = TokenKind::lexer(raw);
        let actual: Vec<TokenKind> = lexer.map(|t| t.unwrap()).filter(|t| t != &TokenKind::Newline).collect();
        assert_eq!(
            actual,
            vec![
                TokenKind::FunctionIdentifier("a".to_string()),
                TokenKind::Assign,
                TokenKind::Integer(1),
                TokenKind::FunctionIdentifier("b".to_string()),
                TokenKind::Assign,
                TokenKind::Integer(2),
                TokenKind::FunctionIdentifier("a".to_string()),
                TokenKind::BinOp(BinaryOperation::Add),
                TokenKind::FunctionIdentifier("b".to_string()),
            ]
        )
    }
}
