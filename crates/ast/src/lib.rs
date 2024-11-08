mod modules;
mod program;
mod token;
mod validate;

#[cfg(feature = "derive")]
mod ast_derive;

#[cfg(feature = "derive")]
pub use rigz_vm::derive::*;

use logos::Logos;
pub use modules::ParsedModule;
// Scope will collide with rigz_vm Scope if this is program::*
pub use program::{
    Assign, Element, Exposed, Expression, FunctionArgument, FunctionDeclaration,
    FunctionDefinition, FunctionSignature, FunctionType, ImportValue, ModuleTraitDefinition, Program, Scope,
    Statement, TraitDefinition,
};

// todo it'd be nice for rigz_vm to not be required by the ast parser, rigz_value?, changes to vm will require lots of downstream crate updates
// Currently the Module trait depends on VM, it would be possible to create a VMModule trait but this causes issues for rigz_ast_derive
pub use rigz_vm::*;
use std::collections::VecDeque;
use std::fmt::Debug;
pub use token::ParsingError;
use token::{Symbol, Token, TokenKind, TokenValue};
pub use validate::*;

#[derive(Debug)]
pub struct Parser<'lex> {
    tokens: VecDeque<Token<'lex>>,
}

// TODO better error messages
pub fn parse(input: &str) -> Result<Program, ParsingError> {
    let mut parser = Parser::prepare(input)?;

    parser.parse()
}

impl<'lex> Parser<'lex> {
    pub fn prepare(input: &'lex str) -> Result<Self, ParsingError> {
        let input = input.trim(); // ensure no trailing newlines to avoid issues in parse_element
        if input.is_empty() {
            return Err(ParsingError::ParseError(
                "Invalid Input, no tokens".to_string(),
            ));
        }

        let mut lexer = TokenKind::lexer(input);
        let mut tokens = VecDeque::new();
        let mut line = 0;
        // todo use relative column numbers
        // let mut offset = 0;
        // let mut start = 0;
        // let mut end = 0;
        loop {
            let kind = match lexer.next() {
                None => break,
                Some(t) => t,
            };
            let span = lexer.span();
            let kind = match kind {
                Ok(t) => t,
                Err(e) => {
                    return Err(ParsingError::ParseError(format!(
                        "Invalid input: {e}, {} {:?}:{}",
                        lexer.slice(),
                        span,
                        line
                    )))
                }
            };

            if kind == TokenKind::Newline {
                line += 1;
            }

            if kind != TokenKind::Comment {
                tokens.push_back(Token { kind, span, line })
            }
        }
        Ok(Parser { tokens })
    }

    pub fn parse(&mut self) -> Result<Program<'lex>, ParsingError> {
        let mut elements = Vec::new();
        while self.has_tokens() {
            elements.push(self.parse_element()?)
        }
        Ok(Program { elements })
    }

    pub fn parse_module_trait_definition(
        &mut self,
    ) -> Result<ModuleTraitDefinition<'lex>, ParsingError> {
        let mut next = self.next_required_token()?;
        let auto_import = if next.kind == TokenKind::Import {
            next = self.next_required_token()?;
            true
        } else {
            false
        };

        if next.kind != TokenKind::Trait {
            return Err(ParsingError::ParseError(format!(
                "Invalid trait, expected trait received {:?}",
                next
            )));
        }

        let definition = self.parse_trait_definition()?;

        Ok(ModuleTraitDefinition {
            definition,
            auto_import,
        })
    }
}

impl<'lex> From<TokenValue<'lex>> for Expression<'lex> {
    #[inline]
    fn from(value: TokenValue<'lex>) -> Self {
        Expression::Value(value.into())
    }
}

impl<'lex> From<&'lex str> for Expression<'lex> {
    #[inline]
    fn from(value: &'lex str) -> Self {
        Expression::Identifier(value)
    }
}

impl<'lex> From<Symbol<'lex>> for Expression<'lex> {
    #[inline]
    fn from(value: Symbol<'lex>) -> Self {
        Expression::Symbol(value.0)
    }
}

impl<'lex, T: Into<Expression<'lex>>> From<T> for Element<'lex> {
    #[inline]
    fn from(value: T) -> Self {
        Element::Expression(value.into())
    }
}

impl<'lex> From<Statement<'lex>> for Element<'lex> {
    #[inline]
    fn from(value: Statement<'lex>) -> Self {
        Element::Statement(value)
    }
}

impl<'lex> Parser<'lex> {
    fn peek_token(&self) -> Option<Token<'lex>> {
        self.tokens.front().cloned()
    }

    pub fn has_tokens(&self) -> bool {
        !self.tokens.is_empty()
    }

    fn peek_required_token(&self) -> Result<Token<'lex>, ParsingError> {
        match self.peek_token() {
            None => Err(Self::eoi_error("peek_required_token")),
            Some(t) => Ok(t),
        }
    }

    fn peek_required_token_eat_newlines(&mut self) -> Result<Token<'lex>, ParsingError> {
        match self.peek_token() {
            None => Err(Self::eoi_error("peek_required_token")),
            Some(t) if t.kind == TokenKind::Newline => {
                self.consume_token(TokenKind::Newline)?;
                self.peek_required_token_eat_newlines()
            }
            Some(t) => Ok(t),
        }
    }

    fn next_token(&mut self) -> Option<Token<'lex>> {
        self.tokens.pop_front()
    }

    fn next_required_token(&mut self) -> Result<Token<'lex>, ParsingError> {
        match self.next_token() {
            None => Err(Self::eoi_error("next_required_token")),
            Some(t) => Ok(t),
        }
    }

    fn consume_token(&mut self, kind: TokenKind<'lex>) -> Result<(), ParsingError> {
        match self.next_token() {
            None => Err(Self::eoi_error_string(format!("expected {}", kind))),
            Some(t) if t.kind != kind => Err(ParsingError::ParseError(format!(
                "expected {}, received {:?}",
                kind, t
            ))),
            Some(_) => Ok(()),
        }
    }

    fn consume_token_eat_newlines(&mut self, kind: TokenKind<'lex>) -> Result<(), ParsingError> {
        match self.next_token() {
            None => Err(Self::eoi_error_string(format!("expected {}", kind))),
            Some(t) if t.kind == TokenKind::Newline => self.consume_token_eat_newlines(kind),
            Some(t) if t.kind != kind => Err(ParsingError::ParseError(format!(
                "expected {}, received {:?}",
                kind, t
            ))),
            Some(_) => Ok(()),
        }
    }

    fn eoi_error(location: &'static str) -> ParsingError {
        ParsingError::ParseError(format!("Unexpected end of input: {location}"))
    }

    fn eoi_error_string(message: String) -> ParsingError {
        ParsingError::ParseError(format!("Unexpected end of input: {message}"))
    }

    fn parse_element(&mut self) -> Result<Element<'lex>, ParsingError> {
        let token = match self.peek_token() {
            None => return Err(Self::eoi_error("parse_element")),
            Some(t) => t,
        };
        let ele = match token.kind {
            TokenKind::Let => {
                self.consume_token(TokenKind::Let)?;
                self.parse_assignment(false)?.into()
            }
            TokenKind::Import => self.parse_import()?.into(),
            TokenKind::Mut => {
                self.consume_token(TokenKind::Mut)?;
                self.parse_assignment(true)?.into()
            }
            TokenKind::Identifier(id) => {
                self.consume_token(TokenKind::Identifier(id))?;
                match self.peek_token() {
                    None => id.into(),
                    Some(t) => match t.kind {
                        TokenKind::Assign => self.parse_assignment_definition(false, id)?.into(),
                        TokenKind::Colon => self.parse_assignment_definition(false, id)?.into(),
                        TokenKind::Increment => {
                            self.consume_token(TokenKind::Increment)?;
                            Statement::BinaryAssignment {
                                lhs: Assign::Identifier(id, false),
                                op: BinaryOperation::Add,
                                expression: Expression::Value(1.into()),
                            }
                            .into()
                        }
                        TokenKind::Decrement => {
                            self.consume_token(TokenKind::Decrement)?;
                            Statement::BinaryAssignment {
                                lhs: Assign::Identifier(id, false),
                                op: BinaryOperation::Sub,
                                expression: Expression::Value(1.into()),
                            }
                            .into()
                        }
                        TokenKind::BinAssign(op) => {
                            self.consume_token(TokenKind::BinAssign(op))?;
                            Statement::BinaryAssignment {
                                lhs: Assign::Identifier(id, false),
                                op,
                                expression: self.parse_expression()?,
                            }
                            .into()
                        }
                        _ => self.parse_identifier_expression(id)?.into(),
                    },
                }
            }
            TokenKind::Type => {
                self.consume_token(TokenKind::Type)?;
                let next = self.next_required_token()?;
                if let TokenKind::TypeValue(name) = next.kind {
                    self.consume_token(TokenKind::Assign)?;
                    Statement::TypeDefinition(name, self.parse_rigz_type(Some(name), false)?).into()
                } else {
                    return Err(ParsingError::ParseError(format!("Invalid type definition expected TypeValue, received {:?}", next)))
                }
            }
            TokenKind::This => {
                self.consume_token(TokenKind::This)?;
                match self.peek_token() {
                    None => Expression::This.into(),
                    Some(t) => match t.kind {
                        TokenKind::Assign => self.parse_this_assignment_definition()?.into(),
                        TokenKind::Increment => {
                            self.consume_token(TokenKind::Increment)?;
                            Statement::BinaryAssignment {
                                lhs: Assign::This,
                                op: BinaryOperation::Add,
                                expression: Expression::Value(1.into()),
                            }
                            .into()
                        }
                        TokenKind::Decrement => {
                            self.consume_token(TokenKind::Decrement)?;
                            Statement::BinaryAssignment {
                                lhs: Assign::This,
                                op: BinaryOperation::Sub,
                                expression: Expression::Value(1.into()),
                            }
                            .into()
                        }
                        TokenKind::BinAssign(op) => {
                            self.consume_token(TokenKind::BinAssign(op))?;
                            Statement::BinaryAssignment {
                                lhs: Assign::This,
                                op,
                                expression: self.parse_expression()?,
                            }
                            .into()
                        }
                        _ => self.parse_this_expression()?.into(),
                    },
                }
            }
            TokenKind::FunctionDef => {
                self.consume_token(TokenKind::FunctionDef)?;
                Statement::FunctionDefinition(self.parse_function_definition(None)?).into()
            }
            TokenKind::Newline => {
                self.consume_token(TokenKind::Newline)?;
                self.parse_element()?
            }
            TokenKind::Trait => {
                self.consume_token(TokenKind::Trait)?;
                Statement::Trait(self.parse_trait_definition()?).into()
            }
            TokenKind::Lifecycle(lifecycle) => {
                self.consume_token(TokenKind::Lifecycle(lifecycle))?;
                match lifecycle {
                    // todo support multiple lifecycles & @test.assert_eq, @test.assert_neq, @test.assert
                    "test" => {
                        let lifecycle = Lifecycle::Test(TestLifecycle);
                        self.consume_token_eat_newlines(TokenKind::FunctionDef)?;
                        Statement::FunctionDefinition(
                            self.parse_function_definition(Some(lifecycle))?,
                        )
                        .into()
                    }
                    _ => {
                        return Err(ParsingError::ParseError(format!(
                            "Lifecycle {lifecycle} is not supported"
                        )))
                    }
                }
            }
            _ => self.parse_expression()?.into(),
        };
        match self.peek_token() {
            None => {}
            Some(t) if t.terminal() => {
                self.consume_token(t.kind)?;
            }
            Some(_) => {}
        }
        Ok(ele)
    }

    fn parse_import(&mut self) -> Result<Statement<'lex>, ParsingError> {
        self.consume_token(TokenKind::Import)?;
        let next = self.next_required_token()?;
        let import_value = match next.kind {
            TokenKind::TypeValue(tv) => {
                ImportValue::TypeValue(tv)
            }
            TokenKind::Value(TokenValue::String(s)) => {
                if s.starts_with("http") {
                    ImportValue::UrlPath(s)
                } else {
                    ImportValue::FilePath(s)
                }
            }
            t => return Err(ParsingError::ParseError(format!(
                "Only type values and string literals are supported in import currently, received {t}"
            ))),
        };
        Ok(Statement::Import(import_value))
    }

    fn parse_expression(&mut self) -> Result<Expression<'lex>, ParsingError> {
        let next = self
            .next_required_token()
            .map_err(|e| ParsingError::ParseError(format!("Invalid Expression {e}")))?;
        let exp = match next.kind {
            TokenKind::Minus => self.parse_unary_expression(UnaryOperation::Neg)?,
            TokenKind::Not => self.parse_unary_expression(UnaryOperation::Not)?,
            TokenKind::Identifier(id) => self.parse_identifier_expression(id)?,
            TokenKind::Value(v) => self.parse_value_expression(v)?,
            TokenKind::This => self.parse_this_expression()?,
            TokenKind::Symbol(s) => self.parse_symbol_expression(s)?,
            TokenKind::Lparen => self.parse_paren_expression()?,
            TokenKind::Lbracket => self.parse_list()?.into(),
            TokenKind::Lcurly => self.parse_map()?,
            TokenKind::Do => Expression::Scope(self.parse_scope()?),
            TokenKind::Unless => Expression::Unless {
                condition: Box::new(self.parse_expression()?),
                then: self.parse_scope()?,
            },
            TokenKind::If => {
                let condition = Box::new(self.parse_expression()?);
                let (then, branch) = self.parse_if_scope()?;
                Expression::If {
                    condition,
                    then,
                    branch,
                }
            }
            TokenKind::TypeValue(type_value) => {
                let type_value = match type_value.parse() {
                    Ok(tv) => tv,
                    Err(e) => {
                        return Err(ParsingError::ParseError(format!(
                            "Failed to read type {:?}",
                            e
                        )))
                    }
                };
                self.consume_token(TokenKind::Period)?;
                let func_name = self.next_required_token()?;
                if let TokenKind::Identifier(func_name) = func_name.kind {
                    Expression::TypeFunctionCall(type_value, func_name, self.parse_args()?)
                } else {
                    return Err(ParsingError::ParseError(format!(
                        "Invalid Token for Type Function Call {:?}",
                        func_name
                    )));
                }
            }
            TokenKind::Error => {
                let next = self.next_required_token()?;
                match next.kind {
                    TokenKind::Value(v) => {
                        Expression::Value(Value::Error(VMError::RuntimeError(v.to_string())))
                    }
                    // todo support identifier, lists, map, and args
                    _ => {
                        return Err(ParsingError::ParseError(format!(
                            "Invalid Token for Error {:?}",
                            next
                        )));
                    }
                }
            }
            _ => {
                return Err(ParsingError::ParseError(format!(
                    "Invalid Token for Expression {:?}",
                    next
                )))
            }
        };
        self.parse_expression_suffix(exp)
    }

    fn parse_expression_suffix(
        &mut self,
        exp: Expression<'lex>,
    ) -> Result<Expression<'lex>, ParsingError> {
        match self.peek_token() {
            None => Ok(exp),
            Some(t) if t.terminal() => {
                // dont consume newline here
                Ok(exp)
            }
            Some(t) => match t.kind {
                TokenKind::Unless => {
                    self.consume_token(TokenKind::Unless)?;
                    Ok(Expression::Unless {
                        condition: Box::new(self.parse_expression()?),
                        then: Scope {
                            elements: vec![exp.into()],
                        },
                    })
                }
                TokenKind::If => {
                    self.consume_token(TokenKind::If)?;
                    Ok(Expression::If {
                        condition: Box::new(self.parse_expression()?),
                        then: Scope {
                            elements: vec![exp.into()],
                        },
                        branch: None,
                    })
                }
                TokenKind::As => {
                    self.consume_token(TokenKind::As)?;
                    Ok(Expression::Cast(Box::new(exp), self.parse_rigz_type(None, false)?))
                }
                TokenKind::BinOp(_) | TokenKind::Pipe | TokenKind::Minus | TokenKind::Period => {
                    Ok(self.parse_inline_expression(exp)?)
                }
                _ => Ok(exp),
            },
        }
    }

    fn parse_assignment(&mut self, mutable: bool) -> Result<Statement<'lex>, ParsingError> {
        let next = self
            .next_required_token()
            .map_err(|e| ParsingError::ParseError(format!("Expected token for assignment: {e}")))?;

        if let TokenKind::Identifier(id) = next.kind {
            self.parse_assignment_definition(mutable, id)
        } else {
            Err(ParsingError::ParseError(format!(
                "Unexpected token for assignment {:?}",
                next
            )))
        }
    }

    fn parse_assignment_definition(
        &mut self,
        mutable: bool,
        id: &'lex str,
    ) -> Result<Statement<'lex>, ParsingError> {
        let token = self.peek_required_token()?;
        let rigz_type = match token.kind {
            TokenKind::Colon => {
                self.consume_token(TokenKind::Colon)?;
                Some(self.parse_rigz_type(None, false)?)
            }
            _ => None
        };
        self.consume_token(TokenKind::Assign)?;
        let lhs = match rigz_type {
            None => {
                Assign::Identifier(id, mutable)
            }
            Some(rigz_type) => {
                Assign::TypedIdentifier(id, mutable, rigz_type)
            }
        };
        Ok(Statement::Assignment {
            lhs,
            expression: self.parse_expression()?,
        })
    }

    fn parse_this_assignment_definition(&mut self) -> Result<Statement<'lex>, ParsingError> {
        self.consume_token(TokenKind::Assign)?;
        Ok(Statement::Assignment {
            lhs: Assign::This,
            expression: self.parse_expression()?,
        })
    }

    fn parse_identifier_expression(
        &mut self,
        id: &'lex str,
    ) -> Result<Expression<'lex>, ParsingError> {
        let args = match self.peek_token() {
            None => return Ok(id.into()),
            Some(next) => match next.kind {
                TokenKind::Value(_)
                | TokenKind::Identifier(_)
                | TokenKind::Symbol(_)
                | TokenKind::Lparen
                | TokenKind::Lcurly
                | TokenKind::Lbracket => self.parse_args()?,
                _ => return self.parse_inline_expression(id),
            },
        };
        Ok(Expression::FunctionCall(id, args))
    }

    fn parse_identifier_expression_skip_inline(
        &mut self,
        id: &'lex str,
    ) -> Result<Expression<'lex>, ParsingError> {
        let args = match self.peek_token() {
            None => return Ok(id.into()),
            Some(next) => match next.kind {
                TokenKind::Period => {
                    self.consume_token(TokenKind::Period)?;
                    return self.parse_instance_call(id.into());
                }
                TokenKind::Value(_)
                | TokenKind::Identifier(_)
                | TokenKind::Symbol(_)
                | TokenKind::Lparen
                | TokenKind::Lcurly
                | TokenKind::Lbracket => self.parse_args()?,
                _ => return Ok(id.into()),
            },
        };
        Ok(Expression::FunctionCall(id, args))
    }

    fn parse_paren_expression(&mut self) -> Result<Expression<'lex>, ParsingError> {
        let expr = self.parse_expression()?;
        self.consume_token(TokenKind::Rparen)?;
        match self.peek_token() {
            None => Ok(expr),
            Some(t) => match t.kind {
                TokenKind::Period => {
                    self.consume_token(TokenKind::Period)?;
                    self.parse_instance_call(expr)
                }
                _ => Ok(expr),
            },
        }
    }

    fn parse_inline_expression<LHS>(&mut self, lhs: LHS) -> Result<Expression<'lex>, ParsingError>
    where
        LHS: Into<Expression<'lex>>,
    {
        let mut res = lhs.into();
        loop {
            match self.next_token() {
                None => break,
                Some(next) if next.terminal() => {
                    // this allows expression suffixes to be handled correctly
                    self.tokens.push_front(next);
                    break;
                }
                Some(next) => match next.kind {
                    TokenKind::Period => {
                        res = self.parse_instance_call(res)?;
                    }
                    TokenKind::BinOp(op) => {
                        res = self.parse_binary_expression(res, op)?
                    }
                    TokenKind::Minus => {
                        let op = BinaryOperation::Sub;
                        res = self.parse_binary_expression(res, op)?
                    }
                    TokenKind::Pipe => {
                        let op = BinaryOperation::BitOr;
                        res = self.parse_binary_expression(res, op)?
                    }
                    TokenKind::And => {
                        let op = BinaryOperation::And;
                        res = self.parse_binary_expression(res, op)?
                    }
                    TokenKind::Comma
                    | TokenKind::Rparen
                    | TokenKind::Rcurly
                    | TokenKind::Rbracket
                    | TokenKind::If
                    | TokenKind::Assign // for maps
                    | TokenKind::Unless
                    | TokenKind::End => {
                        self.tokens.push_front(next);
                        break;
                    }
                    _ => return Err(ParsingError::ParseError(format!("Unexpected {:?} for inline expression", next)))
                },
            }
        }
        Ok(res)
    }

    fn parse_binary_expression(
        &mut self,
        lhs: Expression<'lex>,
        op: BinaryOperation,
    ) -> Result<Expression<'lex>, ParsingError> {
        let next = self.next_required_token()?;
        let rhs = match next.kind {
            // todo values & identifiers need some work, this doesn't handle function calls or instance calls
            // todo we want value expressions evaluated from left to right, can't call parse_value_expression here
            TokenKind::Value(v) => {
                let base = v.into();
                match self.peek_token() {
                    None => base,
                    Some(t) => match t.kind {
                        TokenKind::Period => {
                            self.consume_token(TokenKind::Period)?;
                            self.parse_instance_call(base)?
                        }
                        _ => base,
                    },
                }
            }
            TokenKind::Identifier(id) => self.parse_identifier_expression_skip_inline(id)?,
            TokenKind::Not => self.parse_unary_expression(UnaryOperation::Not)?,
            TokenKind::Minus => self.parse_unary_expression(UnaryOperation::Neg)?,
            TokenKind::Lparen => self.parse_paren_expression()?,
            TokenKind::Lcurly => self.parse_map()?,
            TokenKind::Lbracket => self.parse_list()?.into(),
            TokenKind::Do => Expression::Scope(self.parse_scope()?),
            TokenKind::This => self.parse_this_expression_skip_inline()?,
            _ => {
                return Err(ParsingError::ParseError(format!(
                    "Unexpected {:?} for binary expression: {:?} {}",
                    next, lhs, op
                )))
            }
        };
        Ok(Expression::binary(lhs, op, rhs))
    }

    fn parse_instance_call(
        &mut self,
        lhs: Expression<'lex>,
    ) -> Result<Expression<'lex>, ParsingError> {
        let next = self.next_required_token()?;
        let mut calls = match next.kind {
            TokenKind::Identifier(id) => {
                vec![id]
            }
            // todo support a.0.blah
            _ => {
                return Err(ParsingError::ParseError(format!(
                    "Unexpected {:?} for instance call",
                    next
                )))
            }
        };
        let mut needs_separator = true;
        loop {
            match self.peek_token() {
                None => break,
                Some(t) if t.terminal() => break,
                Some(t) => {
                    if needs_separator {
                        if t.kind == TokenKind::Period {
                            self.consume_token(TokenKind::Period)?;
                            needs_separator = false;
                            continue;
                        } else {
                            break;
                        }
                    } else {
                        if let TokenKind::Identifier(n) = t.kind {
                            self.consume_token(TokenKind::Identifier(n))?;
                            calls.push(n);
                            needs_separator = true;
                            continue;
                        }
                        return Err(ParsingError::ParseError(format!(
                            "Unexpected {:?} for instance call, {:?}.{}",
                            t,
                            lhs,
                            calls.join(".")
                        )));
                    }
                }
            }
        }
        let args = self.parse_args()?;
        Ok(Expression::InstanceFunctionCall(Box::new(lhs), calls, args))
    }

    fn parse_value_expression(
        &mut self,
        value: TokenValue<'lex>,
    ) -> Result<Expression<'lex>, ParsingError> {
        self.parse_inline_expression(value)
    }

    fn parse_this_expression(&mut self) -> Result<Expression<'lex>, ParsingError> {
        self.parse_inline_expression(Expression::This)
    }

    fn parse_this_expression_skip_inline(&mut self) -> Result<Expression<'lex>, ParsingError> {
        match self.peek_token() {
            None => Ok(Expression::This),
            Some(t) => match t.kind {
                TokenKind::Period => {
                    self.consume_token(TokenKind::Period)?;
                    self.parse_instance_call(Expression::This)
                }
                _ => Ok(Expression::This),
            },
        }
    }

    fn parse_symbol_expression(
        &mut self,
        symbol: Symbol<'lex>,
    ) -> Result<Expression<'lex>, ParsingError> {
        self.parse_inline_expression(symbol)
    }

    fn parse_unary_expression(
        &mut self,
        op: UnaryOperation,
    ) -> Result<Expression<'lex>, ParsingError> {
        let exp = self.parse_expression()?;
        Ok(Expression::unary(op, exp))
    }

    fn parse_args(&mut self) -> Result<Vec<Expression<'lex>>, ParsingError> {
        let mut args = Vec::new();
        let mut needs_comma = false;
        loop {
            match self.peek_token() {
                None => break,
                Some(t) if t.terminal() => break,
                Some(t) => match t.kind {
                    TokenKind::Rparen
                    | TokenKind::End
                    // binary operations are handled within parse_expression
                    | TokenKind::BinOp(_)
                    | TokenKind::Pipe
                    | TokenKind::Minus => break,
                    TokenKind::Comma => {
                        self.consume_token(TokenKind::Comma)?;
                        needs_comma = false;
                        continue;
                    }
                    _ if !needs_comma => {
                        args.push(self.parse_expression()?);
                        needs_comma = true
                    }
                    _ => break
                },
            }
        }
        Ok(args)
    }

    fn parse_list(&mut self) -> Result<Vec<Expression<'lex>>, ParsingError> {
        let mut args = Vec::new();
        loop {
            match self.peek_token() {
                None => return Err(ParsingError::ParseError("Missing ]".to_string())),
                Some(t) if t.kind == TokenKind::Rbracket => {
                    self.consume_token(TokenKind::Rbracket)?;
                    break;
                }
                Some(t) if t.kind == TokenKind::Comma => {
                    self.consume_token(TokenKind::Comma)?;
                }
                Some(_) => {
                    args.push(self.parse_expression()?);
                }
            }
        }
        Ok(args)
    }

    fn parse_map(&mut self) -> Result<Expression<'lex>, ParsingError> {
        let mut args = Vec::new();

        loop {
            match self.peek_token() {
                None => return Err(ParsingError::ParseError("Missing }".to_string())),
                Some(t) if t.kind == TokenKind::Rcurly => {
                    self.consume_token(TokenKind::Rcurly)?;
                    break;
                }
                Some(t) if t.kind == TokenKind::Comma => {
                    self.consume_token(TokenKind::Comma)?;
                }
                // todo support {a, b, c, 1, 2, 3}
                Some(_) => {
                    let key = self.parse_expression()?;
                    self.consume_token(TokenKind::Assign)?;
                    let value = self.parse_expression()?;
                    args.push((key, value));
                }
            }
        }
        Ok(Expression::Map(args))
    }

    fn parse_function_definition(
        &mut self,
        lifecycle: Option<Lifecycle>,
    ) -> Result<FunctionDefinition<'lex>, ParsingError> {
        match self.parse_function_declaration()? {
            FunctionDeclaration::Declaration { name, .. } => Err(ParsingError::ParseError(
                format!("Missing body for function definition {name}"),
            )),
            FunctionDeclaration::Definition(mut f) => match lifecycle {
                None => Ok(f),
                Some(l) => {
                    f.lifecycle = Some(l);
                    Ok(f)
                }
            },
        }
    }

    fn parse_function_arguments(&mut self) -> Result<(Vec<FunctionArgument<'lex>>, Option<usize>), ParsingError> {
        let mut args = Vec::new();
        let next = self.peek_required_token()?;
        if next.kind != TokenKind::Lparen {
            return Ok((args, None));
        }

        self.consume_token(TokenKind::Lparen)?;

        let mut var_arg_start = None;
        loop {
            match self.peek_token() {
                None => break,
                Some(t) if t.kind == TokenKind::Rparen => {
                    self.consume_token(TokenKind::Rparen)?;
                    break;
                }
                Some(t) if t.kind == TokenKind::Comma => {
                    self.consume_token(TokenKind::Comma)?;
                    continue;
                }
                Some(_) => {
                    let arg = self.parse_function_argument(var_arg_start.is_some())?;
                    if arg.var_arg {
                        var_arg_start = Some(args.len());
                    }
                    args.push(arg);
                }
            }
        }
        Ok((args, var_arg_start))
    }

    fn check_var_arg(&mut self, existing_var_arg: bool) -> Result<bool, ParsingError> {
        let next = self.peek_required_token()?;
        if next.kind == TokenKind::VariableArgs {
            if existing_var_arg {
                return Err(ParsingError::ParseError(format!("Multiple var args are not allowed {next:?}, everything after after first declaration is considered a var arg")));
            }
            self.consume_token(TokenKind::VariableArgs)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn parse_value(&mut self) -> Result<Value, ParsingError> {
        let token = self.next_required_token()?;
        if let TokenKind::Value(v) = token.kind {
            return Ok(v.into());
        }

        Err(ParsingError::ParseError(format!("Invalid value {token:?}")))
    }

    fn parse_function_argument(&mut self, existing_var_arg: bool) -> Result<FunctionArgument<'lex>, ParsingError> {
        // todo support mut, vm changes required
        let var_arg = self.check_var_arg(existing_var_arg)?;
        let next = self.next_required_token()?;
        if let TokenKind::Identifier(name) = next.kind {
            let mut default_type = true;
            let next = self.peek_required_token()?;
            let mut rigz_type = match next.kind {
                TokenKind::Colon => {
                    self.consume_token(TokenKind::Colon)?;
                    default_type = false;
                    self.parse_rigz_type(None, false)?
                }
                _ => RigzType::Any,
            };

            if rigz_type == RigzType::None {
                return Err(ParsingError::ParseError(format!(
                    "None is not a valid argument type: {next:?}"
                )));
            }

            let default = match self.peek_required_token()?.kind {
                TokenKind::Assign => {
                    self.consume_token(TokenKind::Assign)?;
                    let v = self.parse_value()?;
                    if default_type {
                        rigz_type = v.rigz_type();
                    }
                    Some(v)
                }
                _ => None,
            };

            Ok(FunctionArgument {
                name,
                default,
                function_type: rigz_type.into(),
                var_arg,
            })
        } else {
            Err(ParsingError::ParseError(format!(
                "Invalid Function Argument {:?}",
                next
            )))
        }
    }

    fn parse_return_type(&mut self, mut_self: bool) -> Result<FunctionType, ParsingError> {
        let mut rigz_type = if mut_self {
            RigzType::This
        } else {
            RigzType::default()
        };
        let mut mutable = mut_self;
        match self.peek_token() {
            None => return Err(Self::eoi_error("parse_return_type")),
            Some(t) => {
                if t.kind == TokenKind::Arrow {
                    self.consume_token(TokenKind::Arrow)?;
                    if self.peek_required_token()?.kind == TokenKind::Mut {
                        self.consume_token(TokenKind::Mut)?;
                        mutable = true;
                    }
                    rigz_type = self.parse_rigz_type(None, false)?
                }
            }
        }
        Ok(FunctionType { rigz_type, mutable })
    }

    fn parse_rigz_type(&mut self, name: Option<&'lex str>, paren: bool) -> Result<RigzType, ParsingError> {
        let next = self.next_required_token()?;
        let rigz_type: RigzType = match next.kind {
            // TokenKind::TypeValue("Fn") => {}
            // TokenKind::TypeValue("FnMut") => {}
            TokenKind::TypeValue(id) => {
                match id.parse() {
                    Ok(t) => t,
                    Err(e) => return Err(ParsingError::ParseError(format!("Invalid type {:?}", e))),
                }
            }
            TokenKind::Lbracket => {
                let t = self.peek_required_token()?;
                match t.kind {
                    TokenKind::Rbracket => {
                        self.consume_token(TokenKind::Rbracket)?;
                        RigzType::List(Box::new(RigzType::default()))
                    },
                    TokenKind::TypeValue(_) => {
                        let l = RigzType::List(Box::new(self.parse_rigz_type(None, paren)?));
                        self.consume_token(TokenKind::Rbracket)?;
                        l
                    }
                    _ => return Err(ParsingError::ParseError(format!("Invalid list type {:?}", t))),
                }
            }
            TokenKind::Lcurly => {
                let mut key_type = None;
                let mut value_type = None;
                let mut custom_type = None;
                loop {
                    let t = self.peek_required_token()?;
                    if t.terminal() {
                        self.consume_token(t.kind)?;
                        continue
                    }
                    match t.kind {
                        TokenKind::Identifier(_) if name.is_some() => {
                            custom_type = Some(self.parse_custom_type(name.unwrap())?);
                            break
                        }
                        TokenKind::Rcurly => {
                            self.consume_token(TokenKind::Rcurly)?;
                            break;
                        },
                        TokenKind::TypeValue(_) if key_type.is_none() => {
                            key_type = Some(self.parse_rigz_type(None, paren)?);
                        }
                        TokenKind::Comma if key_type.is_some() => {
                            self.consume_token(TokenKind::Comma)?;
                            value_type = Some(self.parse_rigz_type(None, paren)?);
                        }
                        _ => return Err(ParsingError::ParseError(format!("Invalid map type {:?}", t))),
                    }
                }

                match custom_type {
                    None => {
                        match (key_type, value_type) {
                            (None, None) => RigzType::Map(Box::new(RigzType::default()), Box::new(RigzType::default())),
                            (Some(t), None) => RigzType::Map(Box::new(t.clone()), Box::new(t)),
                            (Some(k), Some(v)) => RigzType::Map(Box::new(k), Box::new(v)),
                            _ => unreachable!()
                        }
                    }
                    Some(t) => RigzType::Custom(t)
                }
            }
            TokenKind::Lparen => {
                let t = self.parse_rigz_type(None, true)?;
                self.consume_token(TokenKind::Rparen)?;
                t
            }
            _ => return Err(ParsingError::ParseError(format!("Invalid type {:?}", next)))
        };

        let rt = match self.peek_token() {
            None => rigz_type,
            Some(t) => {
                match t.kind {
                    TokenKind::Pipe => {
                        RigzType::Union(self.parse_complex_type(rigz_type, true, paren)?)
                    }
                    TokenKind::And => {
                        RigzType::Composite(self.parse_complex_type(rigz_type, false, paren)?)
                    }
                    _ => rigz_type,
                }
            }
        };
        Ok(rt)
    }

    fn parse_custom_type(&mut self, name: &'lex str) -> Result<CustomType, ParsingError> {
        let mut fields = vec![];
        let mut needs_separator = false;
        loop {
            let t = self.peek_required_token()?;
            match t.kind {
                TokenKind::Identifier(id) => {
                    self.consume_token(TokenKind::Identifier(id))?;
                    self.consume_token(TokenKind::Colon)?;
                    fields.push((id.to_string(), self.parse_rigz_type(None, false)?));
                    needs_separator = true;
                }
                TokenKind::Rcurly => {
                    self.consume_token(TokenKind::Rcurly)?;
                    break
                }
                TokenKind::Comma if needs_separator => {
                    self.consume_token(TokenKind::Comma)?;
                    needs_separator = false;
                }
                _ if t.terminal() => {
                    self.consume_token(t.kind)?
                }
                _ => {
                    return Err(ParsingError::ParseError(format!("Invalid token for custom type {t:?}")));
                }
            }
        }
        Ok(CustomType {
            name: name.to_string(),
            fields,
        })
    }

    fn parse_complex_type(&mut self, rigz_type: RigzType, union: bool, paren: bool) -> Result<Vec<RigzType>, ParsingError> {
        if union {
            self.consume_token(TokenKind::Pipe)?;
        } else {
            self.consume_token(TokenKind::And)?;
        }
        let mut complex = vec![rigz_type];
        let mut needs_sep = false;
        loop {
            let t = self.peek_token();
            match t {
                None => break,
                Some(t) if t.terminal() => {
                    self.consume_token(t.kind)?;
                    break
                }
                Some(t) if needs_sep => {
                    if t.kind == TokenKind::Rparen && paren {
                        break
                    }
                    match t.kind {
                        TokenKind::Assign => break,
                        _ => {
                            let separator = if union {
                                TokenKind::Pipe
                            } else {
                                TokenKind::And
                            };
                            if t.kind == separator {
                                self.consume_token(separator)?;
                                needs_sep = false;
                            } else {
                                return Err(ParsingError::ParseError(format!("Unexpected token in {} type - {t:?}", if union {"union"} else {"composite"})))
                            }
                        }
                    }
                }
                Some(_) => {
                    match self.parse_rigz_type(None, paren)? {
                        RigzType::Union(u) if union => {
                            complex.extend(u);
                        }
                        RigzType::Composite(u) if !union => {
                            complex.extend(u);
                        }
                        t => complex.push(t)
                    }
                    needs_sep = true;
                }
            }
        }
        Ok(complex)
    }

    fn parse_function_type_definition(
        &mut self,
        mut_self: bool,
    ) -> Result<FunctionSignature<'lex>, ParsingError> {
        let (arguments, var_args_start) = self.parse_function_arguments()?;
        Ok(FunctionSignature {
            arguments,
            var_args_start,
            return_type: self.parse_return_type(mut_self)?,
            positional: true,
            self_type: None,
        })
    }

    fn parse_scope(&mut self) -> Result<Scope<'lex>, ParsingError> {
        let mut elements = vec![];
        loop {
            let next = self.peek_required_token_eat_newlines()?;
            match next.kind {
                TokenKind::End => {
                    self.consume_token(TokenKind::End)?;
                    break;
                }
                TokenKind::Assign if elements.is_empty() => {
                    self.consume_token(TokenKind::Assign)?;
                    elements.push(self.parse_element()?);
                    break;
                }
                _ => elements.push(self.parse_element()?),
            }
        }
        Ok(Scope { elements })
    }

    fn parse_if_scope(&mut self) -> Result<(Scope<'lex>, Option<Scope<'lex>>), ParsingError> {
        let mut elements = vec![];
        let mut else_encountered = false;
        loop {
            let next = self.peek_required_token()?;
            match next.kind {
                TokenKind::End => {
                    self.consume_token(TokenKind::End)?;
                    break;
                }
                TokenKind::Else => {
                    self.consume_token(TokenKind::Else)?;
                    else_encountered = true;
                    break;
                }
                TokenKind::Assign if elements.is_empty() => {
                    self.consume_token(TokenKind::Assign)?;
                    elements.push(self.parse_element()?);
                    break;
                }
                TokenKind::Newline => {
                    // parse_element eats NewLines so we have to handle that here for valid end
                    self.consume_token(TokenKind::Newline)?;
                }
                _ => elements.push(self.parse_element()?),
            }
        }
        let branch = if else_encountered {
            Some(self.parse_scope()?)
        } else {
            None
        };
        Ok((Scope { elements }, branch))
    }

    fn parse_trait_definition(&mut self) -> Result<TraitDefinition<'lex>, ParsingError> {
        let next = self.next_required_token()?;
        let name = if let TokenKind::TypeValue(name) = next.kind {
            name
        } else {
            return Err(ParsingError::ParseError(format!(
                "Invalid trait, expected trait name received {:?}",
                next
            )));
        };

        let functions = self.parse_trait_declarations()?;
        self.consume_token(TokenKind::End)?;
        Ok(TraitDefinition { name, functions })
    }

    fn parse_trait_declarations(&mut self) -> Result<Vec<FunctionDeclaration<'lex>>, ParsingError> {
        let mut all = Vec::new();
        loop {
            let next = self.peek_required_token_eat_newlines()?;
            match next.kind {
                TokenKind::End => break,
                TokenKind::FunctionDef => {
                    self.consume_token(TokenKind::FunctionDef)?;
                    let def = self.peek_required_token()?;
                    match def.kind {
                        TokenKind::Mut | TokenKind::TypeValue(_) | TokenKind::Identifier(_) => {
                            all.push(self.parse_function_declaration()?)
                        }
                        _ => {
                            return Err(ParsingError::ParseError(format!("Invalid Token in trait declarations {:?}, expected Function Definition or Declaration", def)))
                        }
                    }
                }
                // todo support type definitions here too
                _ => {
                    return Err(ParsingError::ParseError(format!(
                        "Invalid Token in trait declarations {:?}, expected fn or end",
                        next
                    )))
                }
            }
        }
        Ok(all)
    }

    fn parse_function_declaration(&mut self) -> Result<FunctionDeclaration<'lex>, ParsingError> {
        let next = self.peek_required_token()?;
        match next.kind {
            TokenKind::Mut => {
                self.consume_token(TokenKind::Mut)?;
                let next = self.next_required_token()?;

                if let TokenKind::TypeValue(tv) = next.kind {
                    self.parse_typed_function_declaration(Some(tv), true)
                } else {
                    Err(ParsingError::ParseError(format!(
                        "Invalid Token after fn mut {:?}, expected Type",
                        next
                    )))
                }
            }
            TokenKind::TypeValue(tv) => {
                self.consume_token(TokenKind::TypeValue(tv))?;
                self.parse_typed_function_declaration(Some(tv), false)
            }
            TokenKind::Identifier(_) => self.parse_typed_function_declaration(None, false),
            _ => Err(ParsingError::ParseError(format!(
                "Invalid Token in function declaration {:?}, expected mut, Type, or function name",
                next
            ))),
        }
    }

    fn parse_typed_function_declaration(
        &mut self,
        rigz_type: Option<&'lex str>,
        mutable: bool,
    ) -> Result<FunctionDeclaration<'lex>, ParsingError> {
        let mut is_vm = false;
        let self_type = match rigz_type {
            Some(rt) => match rt.parse::<RigzType>() {
                Ok(t) => {
                    self.consume_token(TokenKind::Period)?;
                    is_vm = t.is_vm();
                    if is_vm && !mutable {
                        return Err(ParsingError::ParseError(
                            "VM extensions must be mutable".to_string(),
                        ));
                    }
                    Some(FunctionType {
                        rigz_type: t,
                        mutable,
                    })
                }
                Err(e) => {
                    return Err(ParsingError::ParseError(format!(
                        "Invalid type: {} {:?}",
                        rt, e
                    )))
                }
            },
            None => None,
        };
        let next = self.next_required_token()?;

        let name = match next.kind {
            TokenKind::Type => {
                // hack to support type as function name
                "type"
            }
            TokenKind::Identifier(name) => name,
            // todo support nested types, Module.CustomType
            _ => {
                return match rigz_type {
                    Some(rt) => Err(ParsingError::ParseError(format!(
                        "Invalid Token after {} {} {:?}, expected Identifier",
                        if mutable { "fn mut" } else { "fn" },
                        rt,
                        next
                    ))),
                    None => Err(ParsingError::ParseError(format!(
                        "Invalid Token after {} {:?}, expected Identifier",
                        if mutable { "fn mut" } else { "fn" },
                        next
                    ))),
                }
            }
        };
        let mut type_definition = self.parse_function_type_definition(!is_vm && mutable)?;
        type_definition.self_type = self_type;
        let next = self.peek_required_token_eat_newlines()?;
        let dec = match next.kind {
            TokenKind::FunctionDef | TokenKind::End => FunctionDeclaration::Declaration {
                name,
                type_definition,
            },
            _ => FunctionDeclaration::Definition(FunctionDefinition {
                name,
                type_definition,
                body: self.parse_scope()?,
                lifecycle: None,
            }),
        };
        Ok(dec)
    }
}
