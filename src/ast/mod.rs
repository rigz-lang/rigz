mod program;
mod validate;

use crate::token::{LexingError, Token, TokenKind};
use crate::{FunctionArgument, FunctionDefinition};
use logos::Logos;
pub use program::{Element, Expression, Program, Statement};
use rigz_vm::{BinaryOperation, RigzType, UnaryOperation, Value};
use std::collections::VecDeque;
pub use validate::ValidationError;

pub struct Parser<'lex> {
    tokens: VecDeque<Token<'lex>>,
}

impl<'lex> Parser<'lex> {
    pub fn prepare(input: &'lex str) -> Result<Self, LexingError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(LexingError::ParseError(
                "Invalid Input, no tokens".to_string(),
            ));
        }

        let mut lexer = TokenKind::lexer(input);
        // todo switch to lexer.map.collect to avoid reallocating vecdeque
        let mut tokens = VecDeque::new();
        let mut line = 0;
        // todo use relative column numbers
        // let mut start = 0;
        // let mut end = 0;
        loop {
            let kind = match lexer.next() {
                None => break,
                Some(t) => t?,
            };
            let slice = lexer.slice();
            let span = lexer.span();
            if kind == TokenKind::Newline {
                line += 1;
            }
            tokens.push_back(Token {
                kind,
                span,
                slice,
                line,
            })
        }
        Ok(Parser { tokens })
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Next<'lex> {
    Token(Token<'lex>),
    Expression(Expression<'lex>),
}

impl<'lex> Parser<'lex> {
    pub fn parse(&mut self) -> Result<Program<'lex>, LexingError> {
        let mut elements = Vec::new();
        loop {
            match self.next_element()? {
                None => break,
                Some(e) => elements.push(e),
            }
        }
        Ok(Program { elements })
    }

    fn next_element(&mut self) -> Result<Option<Element<'lex>>, LexingError> {
        match self.next_token() {
            None => Ok(None),
            Some(t) if t.kind == TokenKind::Newline => self.next_element(), // semi colons aren't allowed here?
            Some(t) => Ok(Some(self.parse_element(t)?)),
        }
    }

    fn parse_element(&mut self, token: Token<'lex>) -> Result<Element<'lex>, LexingError> {
        let next = self.next_token();
        let v = match token.kind {
            TokenKind::Value(v) => Element::Expression(self.parse_element_value(v, next)?),
            TokenKind::Let => Element::Statement(self.parse_keyword_assign_token(next, false)?),
            TokenKind::Mut => Element::Statement(self.parse_keyword_assign_token(next, true)?),
            TokenKind::Not => Element::Expression(self.parse_unary(UnaryOperation::Not, next)?),
            TokenKind::Minus => Element::Expression(self.parse_unary(UnaryOperation::Neg, next)?),
            TokenKind::FunctionDef => match next {
                None => {
                    return Err(LexingError::ParseError(
                        "fn `identifier` required".to_string(),
                    ))
                }
                Some(t) => {
                    if let TokenKind::Identifier(id) = t.kind {
                        Element::Statement(self.parse_function_definition(id)?)
                    } else {
                        return Err(LexingError::ParseError(format!(
                            "Unexpected token {:?} for function definition",
                            t
                        )));
                    }
                }
            },
            TokenKind::Identifier(id) => match next {
                None => Element::Expression(Expression::Identifier(id)),
                Some(t) if t.kind == TokenKind::Assign => {
                    Element::Statement(self.parse_assignment(id, false)?)
                }
                Some(t) => Element::Expression(self.parse_identifier_expression(id, Some(t))?),
            },
            TokenKind::Lparen => Element::Expression(self.parse_paren_expression(next)?),
            TokenKind::If => Element::Statement(self.parse_if_statement(next)?),
            TokenKind::Unless => Element::Statement(self.parse_unless_statement(next)?),
            TokenKind::Lcurly => Element::Expression(Expression::Map(self.parse_map_token(next)?)),
            TokenKind::Lbracket => {
                Element::Expression(Expression::List(self.parse_list_token(next)?))
            }
            TokenKind::Do => Element::Expression(Expression::Scope(self.parse_scope(next)?)),
            TokenKind::Return => {
                let exp = match next {
                    None => None,
                    Some(t) => Some(self.parse_expression(t)?),
                };
                Element::Statement(Statement::Return(exp))
            }
            unsupported => {
                return Err(LexingError::ParseError(format!(
                    "Unexpected token for parse_element: {:?}",
                    unsupported
                )))
            }
        };
        Ok(v)
    }

    fn parse_element_value(
        &mut self,
        value: Value,
        next: Option<Token<'lex>>,
    ) -> Result<Expression<'lex>, LexingError> {
        let current = Expression::Value(value);
        let v = match next {
            None => current,
            Some(t) if t.terminal() => current,
            Some(next) => match next.kind {
                TokenKind::As => {
                    let rigz_type = self.parse_type()?;
                    Expression::Cast(Box::new(current), rigz_type)
                }
                TokenKind::BinOp(op) => {
                    let after = match self.next_token() {
                        None => {
                            return Err(LexingError::ParseError(format!(
                                "Expected token to complete expression {:?}",
                                current
                            )))
                        }
                        Some(a) => a,
                    };
                    Expression::BinExp(
                        Box::new(current),
                        op,
                        Box::new(self.parse_expression(after)?),
                    )
                }
                TokenKind::Minus => {
                    let after = match self.next_token() {
                        None => {
                            return Err(LexingError::ParseError(format!(
                                "Expected token to complete expression {:?} {:?}",
                                current, next
                            )))
                        }
                        Some(a) => a,
                    };
                    Expression::BinExp(
                        Box::new(current),
                        BinaryOperation::Sub,
                        Box::new(self.parse_expression(after)?),
                    )
                }
                TokenKind::Period => {
                    todo!()
                }
                unsupported => {
                    return Err(LexingError::ParseError(format!(
                        "Unexpected token for value: {:?}",
                        unsupported
                    )))
                }
            },
        };
        Ok(v)
    }

    fn parse_identifier_expression(
        &mut self,
        id: &'lex str,
        next: Option<Token<'lex>>,
    ) -> Result<Expression<'lex>, LexingError> {
        let current = Expression::Identifier(id);
        let v = match next {
            None => current,
            Some(t) if t.terminal() => current,
            Some(next) => {
                match next.kind {
                    TokenKind::As => {
                        let rigz_type = self.parse_type()?;
                        Expression::Cast(Box::new(current), rigz_type)
                    }
                    TokenKind::BinOp(op) => {
                        let after = match self.next_token() {
                            None => {
                                return Err(LexingError::ParseError(format!(
                                    "Expected token to complete expression {:?}",
                                    current
                                )))
                            }
                            Some(a) => a,
                        };
                        Expression::BinExp(
                            Box::new(current),
                            op,
                            Box::new(self.parse_expression(after)?),
                        )
                    }
                    TokenKind::Minus => {
                        let after = match self.next_token() {
                            None => {
                                return Err(LexingError::ParseError(format!(
                                    "Expected token to complete expression {:?} {:?}",
                                    current, next
                                )))
                            }
                            Some(a) => a,
                        };
                        Expression::BinExp(
                            Box::new(current),
                            BinaryOperation::Sub,
                            Box::new(self.parse_expression(after)?),
                        )
                    }
                    TokenKind::Period => {
                        todo!()
                    }
                    // TODO support multiple args, foo [1, 2, 3], {5}, 42
                    TokenKind::Value(v) => {
                        let after = self.next_token();
                        match after {
                            None => Expression::FunctionCall(id, vec![Expression::Value(v)]),
                            Some(t) if t.kind == TokenKind::Comma => Expression::FunctionCall(
                                id,
                                self.parse_expressions(Expression::Value(v))?,
                            ),
                            Some(t) => self.parse_element_value(v, Some(t))?,
                        }
                    }
                    TokenKind::Lcurly => {
                        Expression::FunctionCall(id, vec![Expression::Map(self.parse_map()?)])
                    }
                    TokenKind::Lbracket => {
                        Expression::FunctionCall(id, vec![Expression::List(self.parse_list()?)])
                    }
                    TokenKind::Symbol(s) => {
                        Expression::FunctionCall(id, vec![Expression::Symbol(s)])
                    }
                    TokenKind::Identifier(arg) => {
                        Expression::FunctionCall(id, vec![Expression::Identifier(arg)])
                    }
                    TokenKind::Unless => {
                        let next = self.next_required_token()?;
                        let condition = Box::new(self.parse_expression(next)?);
                        Expression::Unless {
                            condition,
                            then: Program {
                                elements: vec![Element::Expression(Expression::FunctionCall(
                                    id,
                                    vec![],
                                ))],
                            },
                        }
                    }
                    TokenKind::If => {
                        let next = self.next_required_token()?;
                        let condition = Box::new(self.parse_expression(next)?);
                        Expression::If {
                            condition,
                            then: Program {
                                elements: vec![Element::Expression(Expression::FunctionCall(
                                    id,
                                    vec![],
                                ))],
                            },
                            branch: None,
                        }
                    }
                    unsupported => {
                        return Err(LexingError::ParseError(format!(
                            "Unexpected token for parse_identifier_expression: {:?}",
                            unsupported
                        )))
                    }
                }
            }
        };
        Ok(v)
    }

    fn parse_expression_argument(
        &mut self,
        next: Token<'lex>,
    ) -> Result<Expression<'lex>, LexingError> {
        match next.kind {
            TokenKind::Value(v) => self.parse_element_value(v, None),
            TokenKind::Identifier(v) => self.parse_identifier_expression(v, None),
            TokenKind::Lcurly => Ok(Expression::Map(self.parse_map()?)),
            TokenKind::Lbracket => Ok(Expression::List(self.parse_list()?)),
            unsupported => Err(LexingError::ParseError(format!(
                "Unexpected token for parse_expression_argument: {:?}",
                unsupported
            ))),
        }
    }

    fn parse_map(&mut self) -> Result<Vec<(Expression<'lex>, Expression<'lex>)>, LexingError> {
        let next = self.next_token();
        self.parse_map_token(next)
    }

    fn parse_map_token(
        &mut self,
        next: Option<Token<'lex>>,
    ) -> Result<Vec<(Expression<'lex>, Expression<'lex>)>, LexingError> {
        let mut results = Vec::new();
        let mut next = match next {
            None => {
                return Err(LexingError::ParseError(
                    "Invalid Map, expected }".to_string(),
                ))
            }
            Some(t) => t,
        };
        loop {
            if next.kind == TokenKind::Rcurly {
                break;
            }
            let key = self.parse_expression_argument(next)?;
            next = match self.next_token() {
                None => {
                    return Err(LexingError::ParseError(
                        "Invalid Map, expected =".to_string(),
                    ))
                }
                Some(t) if t.kind == TokenKind::Assign => match self.next_token() {
                    None => {
                        return Err(LexingError::ParseError(
                            "Invalid Map, expected value".to_string(),
                        ))
                    }
                    Some(t) => t,
                },
                Some(t) if t.kind == TokenKind::Comma => {
                    next = match self.next_token() {
                        None => {
                            return Err(LexingError::ParseError(
                                "Invalid Map, expected value".to_string(),
                            ))
                        }
                        Some(t) => t,
                    };
                    results.push((key.clone(), key));
                    continue;
                }
                Some(t) if t.kind == TokenKind::Rcurly => {
                    results.push((key.clone(), key));
                    break;
                }
                Some(t) => {
                    return Err(LexingError::ParseError(format!(
                        "Invalid Map, expected = got {:?}",
                        t
                    )))
                }
            };

            let value = self.parse_expression_argument(next)?;
            results.push((key, value));
            next = match self.next_token() {
                None => {
                    return Err(LexingError::ParseError(
                        "Invalid Map, expected , or }".to_string(),
                    ))
                }
                Some(t) if t.kind == TokenKind::Comma => match self.next_token() {
                    None => {
                        return Err(LexingError::ParseError(
                            "Invalid Map, expected , or }".to_string(),
                        ))
                    }
                    Some(t) => t,
                },
                Some(t) => t,
            };
        }
        Ok(results)
    }

    fn parse_list(&mut self) -> Result<Vec<Expression<'lex>>, LexingError> {
        let next = self.next_token();
        self.parse_list_token(next)
    }
    fn parse_list_token(
        &mut self,
        token: Option<Token<'lex>>,
    ) -> Result<Vec<Expression<'lex>>, LexingError> {
        let mut results = Vec::new();
        let mut next = match token {
            None => {
                return Err(LexingError::ParseError(
                    "Invalid List, expected Expression, `,`, or `]`".to_string(),
                ))
            }
            Some(t) => t,
        };
        loop {
            if next.kind == TokenKind::Rbracket {
                break;
            }

            if next.kind == TokenKind::Comma {
                next = match self.next_token() {
                    None => {
                        return Err(LexingError::ParseError(
                            "Invalid List, expected value".to_string(),
                        ))
                    }
                    Some(t) => t,
                };
                results.push(self.parse_expression_argument(next)?);
            } else {
                results.push(self.parse_expression_argument(next)?);
            }
            next = match self.next_token() {
                None => {
                    return Err(LexingError::ParseError(
                        "Invalid List, expected value".to_string(),
                    ))
                }
                Some(t) => t,
            }
        }
        Ok(results)
    }

    fn parse_expressions(
        &mut self,
        initial: Expression<'lex>,
    ) -> Result<Vec<Expression<'lex>>, LexingError> {
        let mut values = vec![initial];
        let mut next = match self.next_token() {
            None => return Ok(values),
            Some(t) => t,
        };
        loop {
            if next.kind == TokenKind::Comma {
                next = match self.next_token() {
                    None => break,
                    Some(t) => t,
                };
            }
            values.push(self.parse_expression_argument(next)?);
            next = match self.next_token() {
                None => break,
                Some(t) => t,
            };
        }
        Ok(values)
    }

    fn parse_expression(&mut self, token: Token<'lex>) -> Result<Expression<'lex>, LexingError> {
        let next = self.next_token();
        match token.kind {
            TokenKind::Value(v) => self.parse_element_value(v, next),
            TokenKind::Identifier(v) => self.parse_identifier_expression(v, next),
            TokenKind::Lparen => self.parse_paren_expression(next),
            TokenKind::Lcurly => Ok(Expression::Map(self.parse_map_token(next)?)),
            unsupported => Err(LexingError::ParseError(format!(
                "Unexpected token for parse_expression: {:?}",
                unsupported
            ))),
        }
    }

    fn parse_unary(
        &mut self,
        op: UnaryOperation,
        next: Option<Token<'lex>>,
    ) -> Result<Expression<'lex>, LexingError> {
        let next = match next {
            None => {
                return Err(LexingError::ParseError(format!(
                    "Expected token to complete expression: {:?}",
                    next
                )))
            }
            Some(t) => t,
        };
        Ok(Expression::UnaryExp(
            op,
            Box::new(self.parse_expression(next)?),
        ))
    }

    fn parse_assignment(
        &mut self,
        id: &'lex str,
        mutable: bool,
    ) -> Result<Statement<'lex>, LexingError> {
        let next = match self.next_token() {
            None => {
                return Err(LexingError::ParseError(format!(
                    "Required token to complete assignment of {}",
                    id
                )))
            }
            Some(s) => s,
        };

        Ok(Statement::Assignment {
            name: id,
            mutable,
            expression: self.parse_expression(next)?,
        })
    }

    fn parse_keyword_assign_token(
        &mut self,
        next: Option<Token<'lex>>,
        mutable: bool,
    ) -> Result<Statement<'lex>, LexingError> {
        let name = match next {
            None => {
                return Err(LexingError::ParseError(format!(
                    "Required token to complete {} assignment",
                    if mutable { "mutable" } else { "immutable" }
                )))
            }
            Some(t) => {
                if let TokenKind::Identifier(id) = t.kind {
                    id
                } else {
                    return Err(LexingError::ParseError(format!(
                        "Unexpected token {:?} for {} assignment",
                        t,
                        if mutable { "mutable" } else { "immutable" }
                    )));
                }
            }
        };

        match self.next_token() {
            None => Err(LexingError::ParseError(format!(
                "Required token to complete {} assignment",
                if mutable { "mutable" } else { "immutable" }
            ))),
            Some(t) if t.kind != TokenKind::Assign => Err(LexingError::ParseError(format!(
                "Unexpected token {:?} for {} assignment, expected =",
                t,
                if mutable { "mutable" } else { "immutable" }
            ))),
            _ => self.parse_assignment(name, mutable),
        }
    }

    fn parse_function_body(
        &mut self,
        id: &'lex str,
        next: Token<'lex>,
    ) -> Result<Vec<Element<'lex>>, LexingError> {
        if next.kind == TokenKind::Assign {
            let next = match self.next_token() {
                None => {
                    return Err(LexingError::ParseError(format!(
                        "fn `expression` required after assignment {}",
                        id
                    )))
                }
                Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline()?,
                Some(t) => t,
            };
            return Ok(vec![Element::Expression(self.parse_expression(next)?)]);
        }

        let mut next = next;
        let mut elements = Vec::new();
        loop {
            if next.kind == TokenKind::End {
                break;
            }

            elements.push(self.parse_element(next)?);
            next = match self.next_token() {
                None => {
                    return Err(LexingError::ParseError(format!(
                        "fn `end` required for {}",
                        id
                    )))
                }
                Some(t) => t,
            };
        }
        Ok(elements)
    }

    fn parse_scope(
        &mut self,
        next: Option<Token<'lex>>,
    ) -> Result<Vec<Element<'lex>>, LexingError> {
        let mut next = match next {
            None => {
                return Err(LexingError::ParseError(
                    "Expected Expression or `end` for scope".to_string(),
                ))
            }
            Some(t) => t,
        };
        let mut elements = Vec::new();
        loop {
            if next.kind == TokenKind::End {
                break;
            }

            elements.push(self.parse_element(next)?);
            next = match self.next_token() {
                None => {
                    return Err(LexingError::ParseError(
                        "`end` required for scope".to_string(),
                    ))
                }
                Some(t) => t,
            };
        }
        Ok(elements)
    }

    fn parse_type(&mut self) -> Result<RigzType, LexingError> {
        let t = match self.next_token() {
            None => return Err(LexingError::ParseError("Missing type".to_string())),
            Some(t) => {
                if let TokenKind::Identifier(id) = t.kind {
                    match id {
                        "Any" => RigzType::Any,
                        "Number" => RigzType::Number,
                        "String" => RigzType::String,
                        "List" => RigzType::List,
                        "Map" => RigzType::Map,
                        "Bool" => RigzType::Bool,
                        "Error" => RigzType::Error,
                        // TODO support custom types
                        unsupported => {
                            return Err(LexingError::ParseError(format!(
                                "Unsupported type: {}",
                                unsupported
                            )))
                        }
                    }
                } else {
                    return Err(LexingError::ParseError("Missing type".to_string()));
                }
            }
        };
        Ok(t)
    }

    fn parse_non_terminal_newline(&mut self) -> Result<Token<'lex>, LexingError> {
        match self.next_token() {
            None => Err(LexingError::ParseError(
                "Expected token after NewLine".to_string(),
            )),
            Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline(),
            Some(t) => Ok(t),
        }
    }

    fn parse_arg(
        &mut self,
        token: Token<'lex>,
    ) -> Result<(FunctionArgument<'lex>, Option<Token<'lex>>), LexingError> {
        let next = match self.next_token() {
            None => {
                return Err(LexingError::ParseError(
                    "Expected complete argument list".to_string(),
                ))
            }
            Some(t) => t,
        };
        if let TokenKind::Identifier(id) = token.kind {
            match next.kind {
                TokenKind::Comma => Ok((
                    FunctionArgument {
                        name: Some(id),
                        default: None,
                        rigz_type: RigzType::Any,
                    },
                    None,
                )),
                TokenKind::Colon => Ok((
                    FunctionArgument {
                        name: Some(id),
                        default: None,
                        rigz_type: self.parse_type()?,
                    },
                    None,
                )),
                _ => Ok((
                    FunctionArgument {
                        name: Some(id),
                        default: None,
                        rigz_type: RigzType::Any,
                    },
                    Some(next),
                )),
            }
        } else {
            Err(LexingError::ParseError(format!(
                "Invalid argument for parse_arg{:?}",
                next
            )))
        }
    }

    fn parse_args(
        &mut self,
        terminal: TokenKind,
    ) -> Result<Vec<FunctionArgument<'lex>>, LexingError> {
        let mut args = Vec::new();
        let mut next = match self.next_token() {
            None => return Err(LexingError::ParseError("Invalid Arguments".to_string())),
            Some(t) => t,
        };
        loop {
            if next.kind == terminal {
                break;
            }

            let (arg, n) = self.parse_arg(next)?;
            args.push(arg);

            next = match n {
                None => match self.next_token() {
                    None => {
                        return Err(LexingError::ParseError(format!(
                            "Invalid arguments, expected {:?}",
                            terminal
                        )))
                    }
                    Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline()?,
                    Some(t) => t,
                },
                Some(t) => t,
            }
        }
        Ok(args)
    }

    /**
    fn foo = expr
    fn foo -> type = expr
    fn foo

    end
    fn foo(arg1: type) -> type
        stmt*
        expr
    end
    fn foo{arg1: type} -> type
        stmt*
        expr
    end
    */
    fn parse_function_definition(&mut self, id: &'lex str) -> Result<Statement<'lex>, LexingError> {
        let next = match self.next_token() {
            None => {
                return Err(LexingError::ParseError(format!(
                    "fn `body` required for {}",
                    id
                )))
            }
            Some(t) => t,
        };

        let stmt = match next.kind {
            TokenKind::Newline => {
                let next = match self.next_token() {
                    None => {
                        return Err(LexingError::ParseError(format!(
                            "fn `body` or `end` required for {}",
                            id
                        )))
                    }
                    Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline()?,
                    Some(t) => t,
                };
                Statement::FunctionDefinition {
                    name: id,
                    type_definition: FunctionDefinition {
                        arguments: vec![],
                        return_type: RigzType::Any,
                        positional: true,
                    },
                    elements: self.parse_function_body(id, next)?,
                }
            }
            TokenKind::Assign => {
                let next = match self.next_token() {
                    None => {
                        return Err(LexingError::ParseError(format!(
                            "fn `expression` required after function assignment {}",
                            id
                        )))
                    }
                    Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline()?,
                    Some(t) => t,
                };
                Statement::FunctionDefinition {
                    name: id,
                    type_definition: FunctionDefinition {
                        arguments: vec![],
                        return_type: RigzType::Any,
                        positional: true,
                    },
                    elements: vec![Element::Expression(self.parse_expression(next)?)],
                }
            }
            TokenKind::Lparen => {
                let arguments = self.parse_args(TokenKind::Rparen)?;
                let mut next = match self.next_token() {
                    None => {
                        return Err(LexingError::ParseError(format!(
                            "fn `expression` or `return type` required after args {}",
                            id
                        )))
                    }
                    Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline()?,
                    Some(t) => t,
                };
                let return_type = if next.kind == TokenKind::Colon {
                    let t = self.parse_type()?;
                    next = match self.next_token() {
                        None => {
                            return Err(LexingError::ParseError(format!(
                                "fn `expression` required after return type {}",
                                id
                            )))
                        }
                        Some(t) if t.kind == TokenKind::Newline => {
                            self.parse_non_terminal_newline()?
                        }
                        Some(t) => t,
                    };
                    t
                } else {
                    RigzType::Any
                };
                Statement::FunctionDefinition {
                    name: id,
                    type_definition: FunctionDefinition {
                        arguments,
                        return_type,
                        positional: true,
                    },
                    elements: self.parse_function_body(id, next)?,
                }
            }
            TokenKind::Lcurly => {
                let arguments = self.parse_args(TokenKind::Rcurly)?;
                let mut next = match self.next_token() {
                    None => {
                        return Err(LexingError::ParseError(format!(
                            "fn `expression` or `return type` required after args {}",
                            id
                        )))
                    }
                    Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline()?,
                    Some(t) => t,
                };
                let return_type = if next.kind == TokenKind::Arrow {
                    let t = self.parse_type()?;
                    next = match self.next_token() {
                        None => {
                            return Err(LexingError::ParseError(format!(
                                "fn `expression` required after return type {}",
                                id
                            )))
                        }
                        Some(t) => t,
                    };
                    t
                } else {
                    RigzType::Any
                };
                Statement::FunctionDefinition {
                    name: id,
                    type_definition: FunctionDefinition {
                        arguments,
                        return_type,
                        positional: false,
                    },
                    elements: self.parse_function_body(id, next)?,
                }
            }
            TokenKind::Arrow => {
                let return_type = self.parse_type()?;
                let next = match self.next_token() {
                    None => {
                        return Err(LexingError::ParseError(format!(
                            "fn `body` or `end` required for {}",
                            id
                        )))
                    }
                    Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline()?,
                    Some(t) => t,
                };
                Statement::FunctionDefinition {
                    name: id,
                    type_definition: FunctionDefinition {
                        arguments: vec![],
                        return_type,
                        positional: true,
                    },
                    elements: self.parse_function_body(id, next)?,
                }
            }
            unsupported => {
                return Err(LexingError::ParseError(format!(
                    "Unexpected token for fn definition {}: {:?}",
                    id, unsupported
                )))
            }
        };
        Ok(stmt)
    }

    fn parse_if_statement(
        &mut self,
        token: Option<Token<'lex>>,
    ) -> Result<Statement<'lex>, LexingError> {
        let mut tokens = match token {
            None => return Err(LexingError::ParseError("Invalid if statement".to_string())),
            Some(n) => VecDeque::from([n]),
        };
        let mut condition = None;
        let mut then = None;
        loop {
            let next = match self.next_token() {
                None => return Err(LexingError::ParseError("Invalid if statement".to_string())),
                Some(t) => t,
            };
            if next.kind == TokenKind::End {
                break;
            }

            if next.kind == TokenKind::Newline && condition.is_none() {
                let next = match tokens.pop_front() {
                    None => continue,
                    Some(t) => t,
                };
                let mut inner = Parser { tokens };
                condition = Some(inner.parse_expression(next)?);
                tokens = VecDeque::new();
                continue;
            }

            if next.kind == TokenKind::Else {
                let mut inner = Parser { tokens };
                then = Some(inner.parse()?);
                tokens = VecDeque::new();
                continue;
            }

            // todo nested ifs
            tokens.push_back(next)
        }

        let condition = match condition {
            None => {
                return Err(LexingError::ParseError(format!(
                    "Invalid if statement: {:?}",
                    tokens
                )))
            }
            Some(c) => c,
        };

        let (then, branch) = match then {
            None => {
                let mut inner = Parser { tokens };
                (inner.parse()?, None)
            }
            Some(then) => {
                let mut inner = Parser { tokens };
                (then, Some(inner.parse()?))
            }
        };

        Ok(Statement::If {
            condition,
            then,
            branch,
        })
    }

    fn parse_unless_statement(
        &mut self,
        token: Option<Token<'lex>>,
    ) -> Result<Statement<'lex>, LexingError> {
        let mut tokens = match token {
            None => {
                return Err(LexingError::ParseError(
                    "Invalid unless statement".to_string(),
                ))
            }
            Some(n) => VecDeque::from([n]),
        };
        let mut condition = None;
        loop {
            let next = match self.next_token() {
                None => {
                    return Err(LexingError::ParseError(
                        "Invalid unless statement".to_string(),
                    ))
                }
                Some(t) => t,
            };
            if next.kind == TokenKind::End {
                break;
            }

            if next.kind == TokenKind::Newline && condition.is_none() {
                let next = match tokens.pop_front() {
                    None => continue,
                    Some(t) => t,
                };
                let mut inner = Parser { tokens };
                condition = Some(inner.parse_expression(next)?);
                tokens = VecDeque::new();
                continue;
            }

            tokens.push_back(next)
        }

        let condition = match condition {
            None => {
                return Err(LexingError::ParseError(format!(
                    "Invalid if statement: {:?}",
                    tokens
                )))
            }
            Some(c) => c,
        };

        let mut inner = Parser { tokens };
        let then = inner.parse()?;

        Ok(Statement::Unless { condition, then })
    }

    fn next_token(&mut self) -> Option<Token<'lex>> {
        self.tokens.pop_front()
    }

    fn peek_next_token(&mut self) -> Option<&Token<'lex>> {
        self.tokens.front()
    }

    // includes new lines
    fn next_required_token(&mut self) -> Result<Token<'lex>, LexingError> {
        match self.next_token() {
            None => Err(LexingError::ParseError(
                "Expected token received end of input".to_string(),
            )),
            Some(t) => Ok(t),
        }
    }

    fn parse_paren_expression(
        &mut self,
        next: Option<Token<'lex>>,
    ) -> Result<Expression<'lex>, LexingError> {
        let mut next = match next {
            None => {
                return Err(LexingError::ParseError(
                    "Invalid Element, expected expression received nothing".to_string(),
                ))
            }
            Some(t) if t.kind == TokenKind::Rparen => {
                return Err(LexingError::ParseError(
                    "Invalid Element, expected expression received )".to_string(),
                ))
            }
            Some(t) => t,
        };

        let mut up_next = vec![Next::Token(next)];
        loop {
            next = self.next_required_token()?;
            if next.kind == TokenKind::Rparen {
                break;
            }
            if next.kind == TokenKind::Lparen {
                let t = self.next_token();
                up_next.push(Next::Expression(self.parse_paren_expression(t)?));
            }
            up_next.push(Next::Token(next));
        }
        Ok(Expression::Parens(Box::new(
            self.parse_expression_from_upcoming(up_next)?,
        )))
    }

    fn parse_expression_from_upcoming(
        &mut self,
        upcoming: Vec<Next<'lex>>,
    ) -> Result<Expression<'lex>, LexingError> {
        let mut tokens = Vec::new();
        let mut current = None;
        for next in upcoming {
            match next {
                Next::Token(t) => tokens.push(t),
                Next::Expression(ex) => match current {
                    None => {
                        let partial = self.parse_tokens_combine_expression(tokens, ex)?;
                        current = Some(partial);
                        tokens = Vec::new();
                    }
                    Some(prev) => {
                        let partial = self.parse_tokens_combine_expressions(prev, tokens, ex)?;
                        current = Some(partial);
                        tokens = Vec::new();
                    }
                },
            }
        }
        match current {
            None => {
                let mut inner = Parser {
                    tokens: tokens.into(),
                };
                let next = inner.next_required_token()?;
                inner.parse_expression(next)
            }
            Some(ex) => {
                if tokens.is_empty() {
                    Ok(ex)
                } else {
                    self.parse_tokens_combine_expression(tokens, ex)
                }
            }
        }
    }

    fn parse_tokens_combine_expression(
        &self,
        tokens: Vec<Token>,
        ex: Expression<'lex>,
    ) -> Result<Expression<'lex>, LexingError> {
        todo!()
    }

    fn parse_tokens_combine_expressions(
        &self,
        lhs: Expression<'lex>,
        tokens: Vec<Token>,
        rhs: Expression<'lex>,
    ) -> Result<Expression<'lex>, LexingError> {
        todo!()
    }
}

// TODO switch this to ParseError for better error messages
pub fn parse(input: &str) -> Result<Program, LexingError> {
    let mut parser = Parser::prepare(input)?;

    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FunctionArgument;
    use rigz_vm::{RigzType, Value};

    macro_rules! test_parse {
        ($($name:ident $input:literal = $expected:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let input = $input;
                    let v = parse(input).expect("Failed to parse input");
                    assert_eq!(v, $expected)
                }
            )*
        };
    }

    macro_rules! test_parse_fail {
        ($($name:ident $input:literal = $expected:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let input = $input;
                    let v = parse(input).err().expect("Successfully parsed invalid input");
                    assert_eq!(v, $expected)
                }
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
                    assert_eq!(v.is_ok(), true, "Parse Failed {:?}", v.unwrap_err());
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
                    assert_eq!(v.is_err(), true);
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
            do_reserved "do = 1",
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
            valid_bin_exp "1 + 2",
            valid_function "fn hello = none",
            valid_function_dollar_sign "fn $ = none",
            outer_paren_func "(foo 1, 2, 3)",
            named_args_in_func "foo a: 1, b: 2, c: 3",
            let_works "let a = 1",
            mut_works "mut a = 1",
            inline_unless_works "a = b unless c",
            unless_works r#"
                unless c
                    c = 42
                end
            "#,
            if_else r#"
                if c
                    return c * 42
                else
                    c = 24
                end
                c * 37
            "#,
        );
    }

    test_parse! {
        symbols "foo :hello" = Program {
            elements: vec![
                Element::Expression(Expression::FunctionCall("foo", vec![Expression::Symbol("hello")]))
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
                        Box::new(Expression::BinExp(
                            Box::new(Expression::Value(Value::Number(1.into()))),
                            BinaryOperation::Add,
                            Box::new(Expression::Value(Value::Number(2.into())))
                        )),
                        BinaryOperation::Mul,
                        Box::new(Expression::Value(Value::Number(3.into())))
                    )
                )
            ]
        },
        complex_parens "1 + (2 * 3)" = Program {
            elements: vec![
                Element::Expression(
                    Expression::BinExp(
                        Box::new(Expression::Value(Value::Number(1.into()))),
                        BinaryOperation::Add,
                        Box::new(Expression::Parens(
                            Box::new(Expression::BinExp(
                                Box::new(Expression::Value(Value::Number(2.into()))),
                                BinaryOperation::Mul,
                                Box::new(Expression::Value(Value::Number(3.into()))))
                            )
                        )
                        )
                    )
                ),
            ]
        },
        multi_complex_parens "1 + (2 * (2 - 4)) / 4" = Program {
            elements: vec![
                Element::Expression(
                    Expression::BinExp(
                        Box::new(Expression::BinExp(
                        Box::new(Expression::Value(Value::Number(1.into()))),
                        BinaryOperation::Add,
                        Box::new(Expression::Parens(
                            Box::new(Expression::BinExp(
                                Box::new(Expression::Value(Value::Number(2.into()))),
                                BinaryOperation::Mul,
                                Box::new(Expression::Parens(
                                    Box::new(Expression::BinExp(
                                        Box::new(Expression::Value(Value::Number(2.into()))),
                                        BinaryOperation::Sub,
                                        Box::new(Expression::Value(Value::Number(4.into()))))
                                    ))))
                                )
                            )
                        )
                    )),
                        BinaryOperation::Div,
                        Box::new(
                            Expression::Value(Value::Number(4.into()))
                        )
                    )
                )
            ]
        },
        list "[1, '2', {3}]" = Program {
            elements: vec![
                Element::Expression(
                    Expression::List(
                        vec![
                            Expression::Value(Value::Number(1.into())),
                            Expression::Value(Value::String("2".to_string())),
                            Expression::Map(vec![(Expression::Value(Value::Number(3.into())), Expression::Value(Value::Number(3.into())))]),
                        ]
                    )
                )
            ]
        },
        assign "a = 7 - 0" = Program {
            elements: vec![
                Element::Statement(Statement::Assignment {
                    name: "a",
                    expression: Expression::BinExp(
                        Box::new(Expression::Value(Value::Number(7.into()))),
                        BinaryOperation::Sub,
                        Box::new(Expression::Value(Value::Number(0.into())))
                    ),
                    mutable: false,
                })
            ]
        },
        // assign_add "a = 1 + 2; a + 2" = Value::Number(5.into()),
        // unary_not "!1" = Value::Number(Number::Int(!1)),
        // vm_register "__VM.get_register 0" = Value::None,
        define_function_oneline r#"
            fn hello = "hi there"
            hello"# = Program {
            elements: vec![
                Element::Statement(Statement::FunctionDefinition {
                    name: "hello",
                    type_definition: FunctionDefinition {
                        arguments: vec![],
                        positional: true,
                        return_type: RigzType::Any
                    },
                    elements: vec![
                        Element::Expression(Expression::Value(Value::String("hi there".to_string())))
                    ],
                }),
                Element::Expression(Expression::Identifier("hello"))
            ]
        },
        define_function_oneishline r#"
            fn hello
                = "hi there"
            hello"# = Program {
            elements: vec![
                Element::Statement(Statement::FunctionDefinition {
                    name: "hello",
                    type_definition: FunctionDefinition {
                        arguments: vec![],
                        positional: true,
                        return_type: RigzType::Any
                    },
                    elements: vec![
                        Element::Expression(Expression::Value(Value::String("hi there".to_string())))
                    ],
                }),
                Element::Expression(Expression::Identifier("hello"))
            ]
        },
        define_function r#"
            fn hello
                "hi there"
            end
            hello"# = Program {
            elements: vec![
                Element::Statement(Statement::FunctionDefinition {
                    name: "hello",
                    type_definition: FunctionDefinition {
                        arguments: vec![],
                        positional: true,
                        return_type: RigzType::Any
                    },
                    elements: vec![
                        Element::Expression(Expression::Value(Value::String("hi there".to_string())))
                    ],
                }),
                Element::Expression(Expression::Identifier("hello"))
            ]
        },
        define_function_typed r#"
            fn hello -> String
                "hi there"
            end
            hello"# = Program {
            elements: vec![
                Element::Statement(Statement::FunctionDefinition {
                    name: "hello",
                    type_definition: FunctionDefinition {
                        arguments: vec![],
                        positional: true,
                        return_type: RigzType::String
                    },
                    elements: vec![
                        Element::Expression(Expression::Value(Value::String("hi there".to_string())))
                    ],
                }),
                Element::Expression(Expression::Identifier("hello"))
            ]
        },
        define_function_typed_oneish_line r#"
            fn hello -> String
                = "hi there"
            hello"# = Program {
            elements: vec![
                Element::Statement(Statement::FunctionDefinition {
                    name: "hello",
                    type_definition: FunctionDefinition {
                        arguments: vec![],
                        positional: true,
                        return_type: RigzType::String
                    },
                    elements: vec![
                        Element::Expression(Expression::Value(Value::String("hi there".to_string())))
                    ],
                }),
                Element::Expression(Expression::Identifier("hello"))
            ]
        },
        define_function_args r#"
            fn add(a, b, c)
              a + b + c
            end
            add 1, 2, 3"# = Program {
            elements: vec![
                Element::Statement(Statement::FunctionDefinition {
                    name: "add",
                    type_definition: FunctionDefinition {
                        positional: true,
                        arguments: vec![
                            FunctionArgument {
                                name: Some("a"),
                                default: None,
                                rigz_type: RigzType::Any,
                            },
                            FunctionArgument {
                                name: Some("b"),
                                default: None,
                                rigz_type: RigzType::Any,
                            },
                            FunctionArgument {
                                name: Some("c"),
                                default: None,
                                rigz_type: RigzType::Any,
                            },
                        ],
                        return_type: RigzType::Any
                    },
                    elements: vec![
                        Element::Expression(Expression::BinExp(
                            Box::new(Expression::Identifier("a")),
                            BinaryOperation::Add,
                            Box::new(Expression::BinExp(
                                    Box::new(Expression::Identifier("b")),
                                    BinaryOperation::Add,
                                    Box::new(Expression::Identifier("c")))
                        )))
                    ],
                }),
                Element::Expression(Expression::FunctionCall("add", vec![Expression::Value(Value::Number(1.into())), Expression::Value(Value::Number(2.into())), Expression::Value(Value::Number(3.into()))]))
            ]
        },
        define_function_named_args r#"
            fn add{a, b, c}
              a + b + c
            end
            v = {a = 1, b = 2, c = 3}
            add v"# = Program {
            elements: vec![
                Element::Statement(Statement::FunctionDefinition {
                    name: "add",
                    type_definition: FunctionDefinition {
                        positional: false,
                        arguments: vec![
                            FunctionArgument {
                                name: Some("a"),
                                default: None,
                                rigz_type: RigzType::Any,
                            },
                            FunctionArgument {
                                name: Some("b"),
                                default: None,
                                rigz_type: RigzType::Any,
                            },
                            FunctionArgument {
                                name: Some("c"),
                                default: None,
                                rigz_type: RigzType::Any,
                            },
                        ],
                        return_type: RigzType::Any
                    },
                    elements: vec![
                        Element::Expression(Expression::BinExp(
                            Box::new(Expression::Identifier("a")),
                            BinaryOperation::Add,
                            Box::new(Expression::BinExp(
                                    Box::new(Expression::Identifier("b")),
                                    BinaryOperation::Add,
                                    Box::new(Expression::Identifier("c")))
                        )))                    ],
                }),
                Element::Statement(Statement::Assignment {
                    name: "v",
                    mutable: false,
                    expression: Expression::Map(vec![(Expression::Identifier("a"), Expression::Value(Value::Number(1.into()))), (Expression::Identifier("b"), Expression::Value(Value::Number(2.into()))), (Expression::Identifier("c"), Expression::Value(Value::Number(3.into())))]),
                }),
                Element::Expression(Expression::FunctionCall("add", vec![Expression::Identifier("v")]))
            ]
        },
    }
}
