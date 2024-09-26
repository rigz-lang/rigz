use crate::token::{LexingError, ParseError, Token, TokenKind};
use crate::{FunctionArgument, FunctionDefinition};
use logos::{Lexer, Logos};
use rigz_vm::{BinaryOperation, RigzType, UnaryOperation, Value};

pub struct Parser<'lex> {
    lexer: Lexer<'lex, TokenKind<'lex>>,
    line: usize,
}

impl<'lex> Parser<'lex> {
    pub fn create(input: &'lex str) -> Self {
        Parser {
            lexer: TokenKind::lexer(input.trim()),
            line: 0
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Program<'lex> {
    pub elements: Vec<Element<'lex>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Element<'lex> {
    Statement(Statement<'lex>),
    Expression(Expression<'lex>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<'lex> {
    Assignment {
        name: &'lex str,
        mutable: bool,
        expression: Expression<'lex>,
    },
    FunctionDefinition {
        name: &'lex str,
        type_definition: FunctionDefinition<'lex>,
        elements: Vec<Element<'lex>>,
    },
    // import, exports
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression<'lex> {
    Value(Value<'lex>),
    List(Vec<Expression<'lex>>),
    Map(Vec<(Expression<'lex>, Expression<'lex>)>),
    Identifier(&'lex str),
    BinExp(
        Box<Expression<'lex>>,
        BinaryOperation,
        Box<Expression<'lex>>,
    ),
    UnaryExp(UnaryOperation, Box<Expression<'lex>>),
    FunctionCall(&'lex str, Vec<Expression<'lex>>),
    InstanceFunctionCall(Box<Expression<'lex>>, Vec<&'lex str>, Vec<Expression<'lex>>),
    Scope(Vec<Element<'lex>>),
    Cast(Box<Expression<'lex>>, RigzType),
    Parens(Box<Expression<'lex>>),
    Symbol(&'lex str),
}

impl<'lex> Parser<'lex> {
    pub fn parse(&mut self) -> Result<Program<'lex>, ParseError> {
        let mut elements = Vec::new();
        loop {
            match self.next_element() {
                Ok(None) => break,
                Ok(Some(e)) => elements.push(e),
                Err(e) => {
                    // TODO are span and slice correct?
                    return Err(ParseError(e, self.line, self.lexer.span(), self.lexer.slice().to_string()))
                },
            }
        }
        Ok(Program { elements })
    }

    fn next_element(&mut self) -> Result<Option<Element<'lex>>, LexingError> {
        match self.next_token() {
            Ok(None) => Ok(None),
            Ok(Some(t)) if t.kind == TokenKind::Newline => self.next_element(), // semi colons aren't allowed here?
            Ok(Some(t)) => Ok(Some(self.parse_element(t)?)),
            Err(e) => Err(e)
        }
    }

    fn parse_element_value(&mut self, value: Value<'lex>, next: Option<Token<'lex>>) -> Result<Expression<'lex>, LexingError> {
        let current = Expression::Value(value);
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
                        let after = match self.next_token()? {
                            None => return Err(LexingError::ParseError(format!("Expected token to complete expression {:?}", current))),
                            Some(a) => a,
                        };
                        Expression::BinExp(Box::new(current), op, Box::new(self.parse_expression(after)?))
                    }
                    TokenKind::Minus => {
                        let after = match self.next_token()? {
                            None => return Err(LexingError::ParseError(format!("Expected token to complete expression {:?} {:?}", current, next))),
                            Some(a) => a,
                        };
                        Expression::BinExp(Box::new(current), BinaryOperation::Sub, Box::new(self.parse_expression(after)?))
                    }
                    TokenKind::Period => {
                        todo!()
                    }
                    unsupported => return Err(LexingError::ParseError(format!("Unexpected token for value: {:?}", unsupported)))
                }
            }
        };
        Ok(v)
    }

    fn parse_identifier_expression(&mut self, id: &'lex str, next: Option<Token<'lex>>) -> Result<Expression<'lex>, LexingError> {
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
                        let after = match self.next_token()? {
                            None => return Err(LexingError::ParseError(format!("Expected token to complete expression {:?}", current))),
                            Some(a) => a,
                        };
                        Expression::BinExp(Box::new(current), op, Box::new(self.parse_expression(after)?))
                    }
                    TokenKind::Minus => {
                        let after = match self.next_token()? {
                            None => return Err(LexingError::ParseError(format!("Expected token to complete expression {:?} {:?}", current, next))),
                            Some(a) => a,
                        };
                        Expression::BinExp(Box::new(current), BinaryOperation::Sub, Box::new(self.parse_expression(after)?))
                    }
                    TokenKind::Period => {
                        todo!()
                    }
                    // TODO support multiple args, foo [1, 2, 3], {5}, 42
                    TokenKind::Value(v) => {
                        let after = self.next_token()?;
                        match after {
                            None => Expression::FunctionCall(id, vec![Expression::Value(v)]),
                            Some(t) if t.kind == TokenKind::Comma => {
                                Expression::FunctionCall(id, self.parse_expressions(Expression::Value(v))?)
                            }
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
                    unsupported => return Err(LexingError::ParseError(format!("Unexpected token for parse_identifier_expression: {:?}", unsupported)))
                }
            }
        };
        Ok(v)
    }

    fn parse_expression_argument(&mut self, next: Token<'lex>) -> Result<Expression<'lex>, LexingError> {
        match next.kind {
            TokenKind::Value(v) => self.parse_element_value(v, None),
            TokenKind::Identifier(v) => self.parse_identifier_expression(v, None),
            TokenKind::Lcurly => Ok(Expression::Map(self.parse_map()?)),
            TokenKind::Lbracket => Ok(Expression::List(self.parse_list()?)),
            unsupported => Err(LexingError::ParseError(format!("Unexpected token for parse_expression_argument: {:?}", unsupported)))
        }
    }

    fn parse_map(&mut self) -> Result<Vec<(Expression<'lex>, Expression<'lex>)>, LexingError> {
        let next = self.next_token()?;
        self.parse_map_token(next)
    }

    fn parse_map_token(&mut self, next: Option<Token<'lex>>) -> Result<Vec<(Expression<'lex>, Expression<'lex>)>, LexingError> {
        let mut results = Vec::new();
        let mut next = match next {
            None => return Err(LexingError::ParseError("Invalid Map, expected }".to_string())),
            Some(t) => t
        };
        loop {
            if next.kind == TokenKind::Rcurly {
                break
            }
            let key = self.parse_expression_argument(next)?;
            next = match self.next_token()? {
                None => return Err(LexingError::ParseError("Invalid Map, expected =".to_string())),
                Some(t) if t.kind == TokenKind::Assign => {
                    match self.next_token()? {
                        None => return Err(LexingError::ParseError("Invalid Map, expected value".to_string())),
                        Some(t) => t
                    }
                }
                Some(t) if t.kind == TokenKind::Comma => {
                    next = match self.next_token()? {
                        None => return Err(LexingError::ParseError("Invalid Map, expected value".to_string())),
                        Some(t) => t
                    };
                    results.push((key.clone(), key));
                    continue
                }
                Some(t) if t.kind == TokenKind::Rcurly => {
                    results.push((key.clone(), key));
                    break
                },
                Some(t) => return Err(LexingError::ParseError(format!("Invalid Map, expected = got {:?}", t))),
            };

            let value = self.parse_expression_argument(next)?;
            results.push((key, value));
            next = match self.next_token()? {
                None => return Err(LexingError::ParseError("Invalid Map, expected , or }".to_string())),
                Some(t) if t.kind == TokenKind::Comma => {
                    match self.next_token()? {
                        None => return Err(LexingError::ParseError("Invalid Map, expected , or }".to_string())),
                        Some(t) => t
                    }
                }
                Some(t) => t
            };
        }
        Ok(results)
    }

    fn parse_list(&mut self) -> Result<Vec<Expression<'lex>>, LexingError> {
        let next = self.next_token()?;
        self.parse_list_token(next)
    }
    fn parse_list_token(&mut self, token: Option<Token<'lex>>) -> Result<Vec<Expression<'lex>>, LexingError> {
        let mut results = Vec::new();
        let mut next = match token {
            None => return Err(LexingError::ParseError("Invalid List, expected Expression, `,`, or `]`".to_string())),
            Some(t) => t
        };
        loop {
            if next.kind == TokenKind::Rbracket {
                break
            }

            if next.kind == TokenKind::Comma {
                next = match self.next_token()? {
                    None => return Err(LexingError::ParseError("Invalid List, expected value".to_string())),
                    Some(t) => t
                };
                results.push(self.parse_expression_argument(next)?);
            } else {
                results.push(self.parse_expression_argument(next)?);
            }
            next = match self.next_token()? {
                None => return Err(LexingError::ParseError("Invalid List, expected value".to_string())),
                Some(t) => t
            }
        }
        Ok(results)
    }

    fn parse_expressions(&mut self, initial: Expression<'lex>) -> Result<Vec<Expression<'lex>>, LexingError> {
        let mut values = vec![initial];
        let mut next = match self.next_token()? {
            None => return Ok(values),
            Some(t) => t,
        };
        loop {
            if next.kind == TokenKind::Comma {
                next = match self.next_token()? {
                    None => break,
                    Some(t) => t,
                };
            }
            values.push(self.parse_expression_argument(next)?);
            next = match self.next_token()? {
                None => break,
                Some(t) => t,
            };
        }
        Ok(values)
    }

    fn parse_expression(&mut self, token: Token<'lex>) -> Result<Expression<'lex>, LexingError> {
        let next = self.next_token()?;
        let ex = match token.kind {
            TokenKind::Value(v) => self.parse_element_value(v, next)?,
            TokenKind::Identifier(v) => self.parse_identifier_expression(v, next)?,
            unsupported => return Err(LexingError::ParseError(format!("Unexpected token for parse_expression: {:?}", unsupported)))
        };
        Ok(ex)
    }

    fn parse_unary(&mut self, op: UnaryOperation, next: Option<Token<'lex>>) -> Result<Expression<'lex>, LexingError> {
        let next = match next {
            None => return Err(LexingError::ParseError(format!("Expected token to complete expression: {:?}", next))),
            Some(t) => t,
        };
        Ok(Expression::UnaryExp(op, Box::new(self.parse_expression(next)?)))
    }

    fn parse_assignment(&mut self, id: &'lex str, mutable: bool) -> Result<Statement<'lex>, LexingError> {
        let next = match self.next_token()? {
            None => return Err(LexingError::ParseError(format!("Required token to complete assignment of {}", id))),
            Some(s) => s
        };

        Ok(Statement::Assignment {
            name: id,
            mutable,
            expression: self.parse_expression(next)?,
        })
    }

    fn parse_keyword_assign_token(&mut self, next: Option<Token<'lex>>, mutable: bool) -> Result<Statement<'lex>, LexingError> {
        let name = match next {
            None => return Err(LexingError::ParseError(format!("Required token to complete {} assignment", if mutable { "mutable" } else { "immutable" } ))),
            Some(t)  => {
                if let TokenKind::Identifier(id) = t.kind {
                    id
                } else {
                    return Err(LexingError::ParseError(format!("Unexpected token {:?} for {} assignment", t, if mutable { "mutable" } else { "immutable" })))
                }
            },
        };

        match self.next_token()? {
            None => Err(LexingError::ParseError(format!("Required token to complete {} assignment", if mutable { "mutable" } else { "immutable" } ))),
            Some(t) if t.kind != TokenKind::Assign => Err(LexingError::ParseError(format!("Unexpected token {:?} for {} assignment, expected =", t, if mutable { "mutable" } else { "immutable" }))),
            _ => self.parse_assignment(name, mutable),
        }
    }

    fn parse_function_body(&mut self, id: &'lex str, next: Token<'lex>) -> Result<Vec<Element<'lex>>, LexingError> {
        if next.kind == TokenKind::Assign {
            let next = match self.next_token()? {
                None => return Err(LexingError::ParseError(format!("fn `expression` required after assignment {}", id))),
                Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline()?,
                Some(t) => t,
            };
            return Ok(vec![
                Element::Expression(self.parse_expression(next)?)
            ])
        }

        let mut next = next;
        let mut elements = Vec::new();
        loop {
            if next.kind == TokenKind::End {
                break;
            }

            elements.push(self.parse_element(next)?);
            next = match self.next_token()? {
                None => return Err(LexingError::ParseError(format!("fn `end` required for {}", id))),
                Some(t) => t
            };
        }
        Ok(elements)
    }

    fn parse_scope(&mut self, next: Option<Token<'lex>>) -> Result<Vec<Element<'lex>>, LexingError> {
        let mut next = match next {
            None => return Err(LexingError::ParseError("Expected Expression or `end` for scope".to_string())),
            Some(t) => t
        };
        let mut elements = Vec::new();
        loop {
            if next.kind == TokenKind::End {
                break;
            }

            elements.push(self.parse_element(next)?);
            next = match self.next_token()? {
                None => return Err(LexingError::ParseError("`end` required for scope".to_string())),
                Some(t) => t
            };
        }
        Ok(elements)
    }

    fn parse_type(&mut self) -> Result<RigzType, LexingError> {
        let t = match self.next_token()? {
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
                        unsupported => return Err(LexingError::ParseError(format!("Unsupported type: {}", unsupported)))
                    }
                } else {
                    return Err(LexingError::ParseError("Missing type".to_string()))
                }
            }
        };
        Ok(t)
    }

    fn parse_non_terminal_newline(&mut self) -> Result<Token<'lex>, LexingError> {
        match self.next_token()? {
            None => Err(LexingError::ParseError("Expected token after NewLine".to_string())),
            Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline(),
            Some(t) => Ok(t)
        }
    }

    fn parse_arg(&mut self, token: Token<'lex>) -> Result<(FunctionArgument<'lex>, Option<Token<'lex>>), LexingError> {
        let next = match self.next_token()? {
            None => return Err(LexingError::ParseError("Expected complete argument list".to_string())),
            Some(t) => t,
        };
        if let TokenKind::Identifier(id) = token.kind {
            match next.kind {
                TokenKind::Comma => {
                    Ok((FunctionArgument {
                        name: Some(id),
                        default: None,
                        rigz_type: RigzType::Any,
                    }, None))
                }
                TokenKind::Colon => {
                    Ok((FunctionArgument {
                        name: Some(id),
                        default: None,
                        rigz_type: self.parse_type()?,
                    }, None))
                }
                _ => Ok((FunctionArgument {
                    name: Some(id),
                    default: None,
                    rigz_type: RigzType::Any,
                }, Some(next)))
            }
        } else {
            Err(LexingError::ParseError(format!("Invalid argument for parse_arg{:?}", next)))
        }
    }

    fn parse_args(&mut self, terminal: TokenKind) -> Result<Vec<FunctionArgument<'lex>>, LexingError> {
        let mut args = Vec::new();
        let mut next = match self.next_token()? {
            None => return Err(LexingError::ParseError("Invalid Arguments".to_string())),
            Some(t) => t
        };
        loop {
            if next.kind == terminal {
                break
            }

            let (arg, n) = self.parse_arg(next)?;
            args.push(arg);

            next = match n {
                None => match self.next_token()? {
                    None => return Err(LexingError::ParseError(format!("Invalid arguments, expected {:?}", terminal))),
                    Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline()?,
                    Some(t) => t
                },
                Some(t) => t
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
        let next = match self.next_token()? {
            None => return Err(LexingError::ParseError(format!("fn `body` required for {}", id))),
            Some(t) => t
        };

        let stmt = match next.kind {
            TokenKind::Newline => {
                let next = match self.next_token()? {
                    None => return Err(LexingError::ParseError(format!("fn `body` or `end` required for {}", id))),
                    Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline()?,
                    Some(t) => t
                };
                Statement::FunctionDefinition {
                    name: id,
                    type_definition: FunctionDefinition { arguments: vec![], return_type: RigzType::Any, positional: true },
                    elements: self.parse_function_body(id, next)?,
                }
            }
            TokenKind::Assign => {
                let next = match self.next_token()? {
                    None => return Err(LexingError::ParseError(format!("fn `expression` required after function assignment {}", id))),
                    Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline()?,
                    Some(t) => t
                };
                Statement::FunctionDefinition {
                    name: id,
                    type_definition: FunctionDefinition { arguments: vec![], return_type: RigzType::Any, positional: true },
                    elements: vec![
                        Element::Expression(self.parse_expression(next)?)
                    ],
                }
            }
            TokenKind::Lparen => {
                let arguments = self.parse_args(TokenKind::Rparen)?;
                let mut next = match self.next_token()? {
                    None => return Err(LexingError::ParseError(format!("fn `expression` or `return type` required after args {}", id))),
                    Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline()?,
                    Some(t) => t
                };
                let return_type = if next.kind == TokenKind::Colon {
                    let t = self.parse_type()?;
                    next = match self.next_token()? {
                        None => return Err(LexingError::ParseError(format!("fn `expression` required after return type {}", id))),
                        Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline()?,
                        Some(t) => t
                    };
                    t
                } else {
                    RigzType::Any
                };
                Statement::FunctionDefinition {
                    name: id,
                    type_definition: FunctionDefinition { arguments, return_type, positional: true },
                    elements: self.parse_function_body(id, next)?,
                }
            }
            TokenKind::Lcurly => {
                let arguments = self.parse_args(TokenKind::Rcurly)?;
                let mut next = match self.next_token()? {
                    None => return Err(LexingError::ParseError(format!("fn `expression` or `return type` required after args {}", id))),
                    Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline()?,
                    Some(t) => t
                };
                let return_type = if next.kind == TokenKind::Arrow {
                    let t = self.parse_type()?;
                    next = match self.next_token()? {
                        None => return Err(LexingError::ParseError(format!("fn `expression` required after return type {}", id))),
                        Some(t) => t
                    };
                    t
                } else {
                    RigzType::Any
                };
                Statement::FunctionDefinition {
                    name: id,
                    type_definition: FunctionDefinition { arguments, return_type, positional: false },
                    elements: self.parse_function_body(id, next)?,
                }
            }
            TokenKind::Arrow => {
                let return_type = self.parse_type()?;
                let next = match self.next_token()? {
                    None => return Err(LexingError::ParseError(format!("fn `body` or `end` required for {}", id))),
                    Some(t) if t.kind == TokenKind::Newline => self.parse_non_terminal_newline()?,
                    Some(t) => t
                };
                Statement::FunctionDefinition {
                    name: id,
                    type_definition: FunctionDefinition { arguments: vec![], return_type, positional: true },
                    elements: self.parse_function_body(id, next)?,
                }
            }
            unsupported => return Err(LexingError::ParseError(format!("Unexpected token for fn definition {}: {:?}", id, unsupported)))
        };
        Ok(stmt)
    }

    fn parse_element(&mut self, token: Token<'lex>) -> Result<Element<'lex>, LexingError> {
        let next = self.next_token()?;
        let v = match token.kind {
            TokenKind::Value(v) => Element::Expression(self.parse_element_value(v, next)?),
            TokenKind::Let => Element::Statement(self.parse_keyword_assign_token(next, false)?),
            TokenKind::Mut => Element::Statement(self.parse_keyword_assign_token(next, true)?),
            TokenKind::Not => Element::Expression(self.parse_unary(UnaryOperation::Not, next)?),
            TokenKind::Minus => Element::Expression(self.parse_unary(UnaryOperation::Neg, next)?),
            TokenKind::FunctionDef => {
                match next {
                    None => return Err(LexingError::ParseError("fn `identifier` required".to_string())),
                    Some(t) => {
                        if let TokenKind::Identifier(id) = t.kind {
                            Element::Statement(self.parse_function_definition(id)?)
                        } else {
                            return Err(LexingError::ParseError(format!("Unexpected token {:?} for function definition", t)))
                        }
                    },
                }
            }
            TokenKind::Identifier(id) => {
                match next {
                    None => Element::Expression(Expression::Identifier(id)),
                    Some(t) if t.kind == TokenKind::Assign => Element::Statement(self.parse_assignment(id, false)?),
                    Some(t) => Element::Expression(self.parse_identifier_expression(id, Some(t))?)
                }
            }
            TokenKind::Lparen => {
                todo!()
            }
            TokenKind::Lcurly => Element::Expression(Expression::Map(self.parse_map_token(next)?)),
            TokenKind::Lbracket => Element::Expression(Expression::List(self.parse_list_token(next)?)),
            TokenKind::Do => Element::Expression(Expression::Scope(self.parse_scope(next)?)),
            unsupported => return Err(LexingError::ParseError(format!("Unexpected token for parse_element: {:?}", unsupported)))
        };
        Ok(v)
    }

    fn next_token(&mut self) -> Result<Option<Token<'lex>>, LexingError> {
        let t = match self.lexer.next() {
            None => None,
            Some(t) => {
                let kind = t?;
                let slice = self.lexer.slice();
                let span = self.lexer.span();
                if kind == TokenKind::Newline {
                    self.line += 1;
                }
                Some(Token { kind, slice, span })
            }
        };
        Ok(t)
    }
}

pub fn parse(input: &str) -> Result<Program, ParseError> {
    let mut parser = Parser::create(input);

    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rigz_vm::{RigzType, Value};
    use crate::FunctionArgument;

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
                    assert_eq!(v.is_ok(), true);
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
        );
    }

    mod valid {
        use super::*;

        test_parse_valid!(
            valid_bin_exp "1 + 2",
            valid_function "fn hello = none",
            outer_paren_func "(foo 1, 2, 3)",
            paren_func "foo(1, 2, 3)",
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
                        Box::new(Expression::Value(Value::Number(1.into()))),
                        BinaryOperation::Add,
                        Box::new(Expression::BinExp(
                                Box::new(Expression::Value(Value::Number(2.into()))),
                                BinaryOperation::Mul,
                                Box::new(Expression::Value(Value::Number(3.into())))))
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
                            ))
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
                Element::Expression(Expression::FunctionCall("add", vec![]))
            ]
        },
    }
}
