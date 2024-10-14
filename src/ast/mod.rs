mod program;
mod validate;

use crate::token::{ParsingError, Symbol, Token, TokenKind, TokenValue};
use crate::{FunctionArgument, FunctionDefinition};
use logos::Logos;
pub use program::{Element, Expression, Program, Scope, Statement};
use rigz_vm::{BinaryOperation, RigzType, UnaryOperation};
use std::collections::VecDeque;
use std::fmt::{format, Debug};
pub use validate::ValidationError;

#[derive(Debug)]
pub struct Parser<'lex> {
    tokens: VecDeque<Token<'lex>>,
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

    fn has_tokens(&self) -> bool {
        !self.tokens.is_empty()
    }

    fn peek_required_token(&self) -> Result<Token<'lex>, ParsingError> {
        match self.peek_token() {
            None => Err(Self::eoi_error("peek_required_token")),
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
            TokenKind::Mut => {
                self.consume_token(TokenKind::Mut)?;
                self.parse_assignment(true)?.into()
            }
            TokenKind::Identifier(id) => {
                self.consume_token(TokenKind::Identifier(id))?;
                match self.peek_token() {
                    None => id.into(),
                    Some(t) if t.kind == TokenKind::Assign => {
                        self.parse_assignment_definition(false, id)?.into()
                    }
                    Some(_) => self.parse_identifier_expression(id)?.into(),
                }
            }
            TokenKind::FunctionDef => self.parse_function_definition()?.into(),
            TokenKind::Newline => {
                self.consume_token(TokenKind::Newline)?;
                self.parse_element()?
            }
            _ => self.parse_expression()?.into(),
        };
        Ok(ele)
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
            TokenKind::Symbol(s) => self.parse_symbol_expression(s)?,
            TokenKind::Lparen => self.parse_paren_expression()?,
            TokenKind::Lbracket => self.parse_list()?,
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
            _ => return Err(ParsingError::ParseError(format!(
                "Invalid Token for Expression {:?}",
                next
            ))),
        };
        self.parse_expression_suffix(exp)
    }

    fn parse_expression_suffix(&mut self, exp: Expression<'lex>) -> Result<Expression<'lex>, ParsingError> {
        match self.peek_token() {
            None => Ok(exp),
            Some(t) if t.terminal() => Ok(exp),
            Some(t) => match t.kind {
                TokenKind::Unless => {
                    self.consume_token(TokenKind::Unless)?;
                    Ok(Expression::Unless {
                        condition: Box::new(self.parse_expression()?),
                        then: Scope {
                            elements: vec![exp.into()],
                        },
                    })
                },
                TokenKind::If => {
                    self.consume_token(TokenKind::If)?;
                    Ok(Expression::If {
                        condition: Box::new(self.parse_expression()?),
                        then: Scope {
                            elements: vec![exp.into()],
                        },
                        branch: None,
                    })
                },
                TokenKind::As => {
                    self.consume_token(TokenKind::As)?;
                    Ok(Expression::Cast(Box::new(exp), self.parse_rigz_type()?))
                }
                TokenKind::Period => {
                    self.consume_token(TokenKind::Period)?;
                    Ok(self.parse_instance_call(exp)?)
                }
                TokenKind::Elvis => {
                    self.consume_token(TokenKind::Elvis)?;
                    Ok(Expression::binary(exp, BinaryOperation::Or, self.parse_expression()?))
                }
                _ => Ok(exp)
            }
        }
    }

    fn parse_assignment(&mut self, mutable: bool) -> Result<Statement<'lex>, ParsingError> {
        let next = self.next_required_token().map_err(|e| {
            ParsingError::ParseError(format!("Expected token for assignment: {e}"))
        })?;

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
        self.consume_token(TokenKind::Assign)?;
        Ok(Statement::Assignment {
            name: id,
            mutable,
            expression: self.parse_expression()?,
        })
    }

    fn parse_identifier_expression(
        &mut self,
        id: &'lex str,
    ) -> Result<Expression<'lex>, ParsingError> {
        let args = match self.peek_token() {
            None => return self.parse_inline_expression(id),
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

    fn parse_paren_expression(&mut self) -> Result<Expression<'lex>, ParsingError> {
        let expr = self.parse_expression()?;
        self.consume_token(TokenKind::Rparen)?;
        Ok(expr)
    }

    fn parse_inline_expression<LHS>(&mut self, lhs: LHS) -> Result<Expression<'lex>, ParsingError>
    where
        LHS: Into<Expression<'lex>>,
    {
        let mut res = lhs.into();
        loop {
            match self.next_token() {
                None => break,
                Some(next) if next.terminal() => break,
                Some(next) => match next.kind {
                    TokenKind::Period => {
                        res = self.parse_instance_call(res)?;
                    },
                    TokenKind::BinOp(op) => {
                        res = self.parse_binary_expression(res, op)?
                    },
                    TokenKind::Minus => {
                        let op = BinaryOperation::Sub;
                        res = self.parse_binary_expression(res, op)?
                    }
                    TokenKind::Pipe => {
                        let op = BinaryOperation::BitOr;
                        res = self.parse_binary_expression(res, op)?
                    }
                    TokenKind::Comma
                    | TokenKind::Rparen
                    | TokenKind::Rcurly
                    | TokenKind::If
                    | TokenKind::Assign // for maps
                    | TokenKind::Unless => {
                        self.tokens.push_front(next);
                        break
                    }
                    _ => return Err(ParsingError::ParseError(format!("Unexpected {:?} for inline expression", next)))
                }
            }
        }
        Ok(res)
    }

    fn parse_binary_expression(&mut self, lhs: Expression<'lex>, op: BinaryOperation) -> Result<Expression<'lex>, ParsingError> {
        let next = self.next_required_token()?;
        let rhs = match next.kind {
            TokenKind::Value(v) => v.into(),
            TokenKind::Not => self.parse_unary_expression(UnaryOperation::Not)?,
            TokenKind::Minus => self.parse_unary_expression(UnaryOperation::Neg)?,
            TokenKind::Identifier(id) => id.into(),
            TokenKind::Lparen => self.parse_paren_expression()?,
            TokenKind::Lcurly => self.parse_map()?,
            TokenKind::Lbracket => self.parse_list()?,
            TokenKind::Do => Expression::Scope(self.parse_scope()?),
            _ => return Err(ParsingError::ParseError(format!("Unexpected {:?} for binary expression: {:?} {}", next, lhs, op)))
        };
        Ok(Expression::binary(lhs, op, rhs))
    }

    fn parse_instance_call(
        &mut self,
        lhs: Expression<'lex>,
    ) -> Result<Expression<'lex>, ParsingError> {
        // a.b.c.d Instance{a,["b, c, d"]}
        // a.b.c.d 1, 2, 3
        let next = self.next_required_token()?;
        let mut calls = match next.kind {
            TokenKind::Identifier(id) => {
                vec![id]
            }
            // todo support a.0
            _ => return Err(ParsingError::ParseError(format!("Unexpected {:?} for instance call", next)))
        };
        //a.b a
        let mut needs_separator = true;
        loop {
            match self.peek_token() {
                None => break,
                Some(t) => {
                    if needs_separator {
                        if t.kind == TokenKind::Period {
                            self.consume_token(TokenKind::Period)?;
                            needs_separator = false;
                            continue
                        } else {
                            break
                        }
                    } else {
                        if let TokenKind::Identifier(n) = t.kind {
                            self.consume_token(TokenKind::Identifier(n))?;
                            calls.push(n);
                            needs_separator = true;
                            continue
                        }
                        return Err(ParsingError::ParseError(format!("Unexpected {:?} for instance call, {:?}.{}", t, lhs, calls.join("."))))
                    }
                }
            }
        }
        Ok(Expression::InstanceFunctionCall(Box::new(lhs), calls, self.parse_args()?))
    }

    fn parse_value_expression(
        &mut self,
        value: TokenValue<'lex>,
    ) -> Result<Expression<'lex>, ParsingError> {
        self.parse_inline_expression(value)
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
        loop {
            match self.peek_token() {
                None => break,
                Some(t) if t.terminal() => {
                    self.consume_token(t.kind.clone())?;
                    break;
                }
                Some(t) if t.kind == TokenKind::Rparen => break,
                Some(t) if t.kind == TokenKind::Comma => {
                    self.consume_token(TokenKind::Comma)?;
                    continue;
                }
                Some(_) => {
                    args.push(self.parse_expression()?);
                }
            }
        }
        Ok(args)
    }

    fn parse_list(&mut self) -> Result<Expression<'lex>, ParsingError> {
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
                    continue;
                }
                Some(_) => {
                    args.push(self.parse_expression()?);
                }
            }
        }
        Ok(Expression::List(args))
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
                    break;
                }
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

    fn parse_function_definition(&mut self) -> Result<Statement<'lex>, ParsingError> {
        self.consume_token(TokenKind::FunctionDef)?;
        let next = self.next_required_token()?;
        if let TokenKind::Identifier(name) = next.kind {
            Ok(Statement::FunctionDefinition {
                name,
                type_definition: self.parse_function_type_definition()?,
                body: self.parse_scope()?,
            })
        } else {
            Err(ParsingError::ParseError(format!(
                "Invalid Function definition {:?}",
                next
            )))
        }
    }

    fn parse_function_arguments(&mut self) -> Result<Vec<FunctionArgument<'lex>>, ParsingError> {
        let mut args = Vec::new();
        let next = self.peek_required_token()?;
        if next.kind != TokenKind::Lparen {
            return Ok(args);
        }

        self.consume_token(TokenKind::Lparen)?;

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
                    args.push(self.parse_function_argument()?);
                }
            }
        }
        Ok(args)
    }

    fn parse_function_argument(&mut self) -> Result<FunctionArgument<'lex>, ParsingError> {
        let next = self.next_required_token()?;
        if let TokenKind::Identifier(name) = next.kind {
            let rigz_type = match self.peek_required_token()?.kind.clone() {
                TokenKind::Colon => {
                    self.consume_token(TokenKind::Colon)?;
                    self.parse_return_type()?
                }
                _ => RigzType::Any,
            };
            Ok(FunctionArgument {
                name: Some(name),
                default: None,
                rigz_type,
            })
        } else {
            Err(ParsingError::ParseError(format!(
                "Invalid Function Argument {:?}",
                next
            )))
        }
    }

    fn parse_return_type(&mut self) -> Result<RigzType, ParsingError> {
        let mut rigz_type = RigzType::Any;
        match self.peek_token() {
            None => return Err(Self::eoi_error("parse_return_type")),
            Some(t) => {
                if t.kind == TokenKind::Arrow {
                    self.consume_token(TokenKind::Arrow)?;
                    rigz_type = self.parse_rigz_type()?
                }
            }
        }
        Ok(rigz_type)
    }

    fn parse_rigz_type(&mut self) -> Result<RigzType, ParsingError> {
        let next = self.next_required_token()?;
        if let TokenKind::TypeValue(id) = next.kind {
            let t = match id {
                "Any" => RigzType::Any,
                "Number" => RigzType::Number,
                "Int" => RigzType::Int,
                "Float" => RigzType::Float,
                "String" => RigzType::String,
                "None" => RigzType::None,
                "Error" => RigzType::Error,
                "List" => RigzType::List,
                "Map" => RigzType::Map,
                "Bool" => RigzType::Bool,
                _ => todo!(),
            };
            Ok(t)
        } else {
            Err(ParsingError::ParseError(format!("Invalid type {:?}", next)))
        }
    }

    fn parse_function_type_definition(&mut self) -> Result<FunctionDefinition<'lex>, ParsingError> {
        Ok(FunctionDefinition {
            arguments: self.parse_function_arguments()?,
            return_type: self.parse_return_type()?,
            positional: true,
        })
    }

    fn parse_scope(&mut self) -> Result<Scope<'lex>, ParsingError> {
        let mut elements = vec![];
        loop {
            let next = self.peek_required_token()?;
            match next.kind.clone() {
                TokenKind::End => {
                    self.consume_token(TokenKind::End)?;
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
        Ok(Scope { elements })
    }

    fn parse_if_scope(&mut self) -> Result<(Scope<'lex>, Option<Scope<'lex>>), ParsingError> {
        let mut elements = vec![];
        let mut else_encountered = false;
        loop {
            let next = self.peek_required_token()?;
            match next.kind.clone() {
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
}

// TODO better error messages
pub fn parse(input: &str) -> Result<Program, ParsingError> {
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
                    let v = parse(input);
                    assert_eq!(v, Ok($expected), "Failed to parse input: {}", input)
                }
            )*
        };
    }

    // macro_rules! test_parse_fail {
    //     ($($name:ident $input:literal = $expected:expr,)*) => {
    //         $(
    //             #[test]
    //             fn $name() {
    //                 let input = $input;
    //                 let v = parse(input).err();
    //                 assert_eq!(v, Some($expected), "Successfully parsed invalid input: {}", input)
    //             }
    //         )*
    //     };
    // }

    macro_rules! test_parse_valid {
        ($($name:ident $input:literal,)*) => {
            $(
                #[test]
                fn $name() {
                    let input = $input;
                    let v = parse(input);
                    assert_eq!(v.is_ok(), true, "Parse Failed {:?} - {}", v.unwrap_err(), input);
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
                    assert_eq!(v.is_err(), true, "Successfully parsed invalid input {}", input);
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
            do_one_line "do = 1 + 2",
            valid_bin_exp "1 + 2",
            valid_function "fn hello = none",
            valid_function_dollar_sign "fn $ = none",
            outer_paren_func "(foo 1, 2, 3)",
            //named_args_in_func "foo a: 1, b: 2, c: 3",
            let_works "let a = 1",
            mut_works "mut a = 1",
            inline_unless_works "a = b unless c",
            instance_methods "a.b.c.d 1, 2, 3",
            // unless_works r#"
            //     unless c
            //         c = 42
            //     end
            // "#,
            // if_else r#"
            //     if c
            //         return c * 42
            //     else
            //         c = 24
            //     end
            //     c * 37
            // "#,
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
                        Box::new(Expression::BinExp(
                            Box::new(Expression::Value(Value::Number(2.into()))),
                            BinaryOperation::Mul,
                            Box::new(Expression::Value(Value::Number(3.into()))))
                        )
                    )
                ),
            ]
        },
        list "[1, '2', {a = 3}]" = Program {
            elements: vec![
                Element::Expression(
                    Expression::List(
                        vec![
                            Expression::Value(Value::Number(1.into())),
                            Expression::Value(Value::String("2".to_string())),
                            Expression::Map(vec![(Expression::Identifier("a"), Expression::Value(Value::Number(3.into())))]),
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
                    body: Scope {
                     elements: vec![
                        Element::Expression(Expression::Value(Value::String("hi there".to_string())))
                    ],
                    }
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
                    body: Scope {
                        elements: vec![
                            Element::Expression(Expression::Value(Value::String("hi there".to_string())))
                        ],
                        }
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
                    body: Scope {
                    elements: vec![
                        Element::Expression(Expression::Value(Value::String("hi there".to_string())))
                    ],
                        }
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
                    body: Scope {
                     elements: vec![
                        Element::Expression(Expression::Value(Value::String("hi there".to_string())))
                        ],
                    }
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
                    body: Scope {
                        elements: vec![
                            Element::Expression(Expression::Value(Value::String("hi there".to_string())))
                        ],
                    }
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
                    body: Scope {
                        elements: vec![
                            Expression::binary(
                                Expression::binary("a".into(), BinaryOperation::Add, "b".into()),
                                BinaryOperation::Add,
                                "c".into()
                            ).into(),
                        ],
                    }
                }),
                Element::Expression(Expression::FunctionCall("add", vec![Expression::Value(Value::Number(1.into())), Expression::Value(Value::Number(2.into())), Expression::Value(Value::Number(3.into()))]))
            ]
        },
        // todo support later
        // define_function_named_args r#"
        //     fn add{a, b, c}
        //       a + b + c
        //     end
        //     v = {a = 1, b = 2, c = 3}
        //     add v"# = Program {
        //     elements: vec![
        //         Element::Statement(Statement::FunctionDefinition {
        //             name: "add",
        //             type_definition: FunctionDefinition {
        //                 positional: false,
        //                 arguments: vec![
        //                     FunctionArgument {
        //                         name: Some("a"),
        //                         default: None,
        //                         rigz_type: RigzType::Any,
        //                     },
        //                     FunctionArgument {
        //                         name: Some("b"),
        //                         default: None,
        //                         rigz_type: RigzType::Any,
        //                     },
        //                     FunctionArgument {
        //                         name: Some("c"),
        //                         default: None,
        //                         rigz_type: RigzType::Any,
        //                     },
        //                 ],
        //                 return_type: RigzType::Any
        //             },
        //             elements: vec![
        //                 Element::Expression(Expression::BinExp(
        //                     Box::new(Expression::Identifier("a")),
        //                     BinaryOperation::Add,
        //                     Box::new(Expression::BinExp(
        //                             Box::new(Expression::Identifier("b")),
        //                             BinaryOperation::Add,
        //                             Box::new(Expression::Identifier("c")))
        //                 )))                    ],
        //         }),
        //         Element::Statement(Statement::Assignment {
        //             name: "v",
        //             mutable: false,
        //             expression: Expression::Map(vec![(Expression::Identifier("a"), Expression::Value(Value::Number(1.into()))), (Expression::Identifier("b"), Expression::Value(Value::Number(2.into()))), (Expression::Identifier("c"), Expression::Value(Value::Number(3.into())))]),
        //         }),
        //         Element::Expression(Expression::FunctionCall("add", vec![Expression::Identifier("v")]))
        //     ]
        // },
    }

    mod debug {
        use super::*;

        test_parse! {
            multi_complex_parens "1 + (2 * (2 - 4)) / 4" = Program {
                elements: vec![
                    Element::Expression(
                        Expression::BinExp(
                            Box::new(Expression::BinExp(
                            Box::new(Expression::Value(Value::Number(1.into()))),
                            BinaryOperation::Add,
                            Box::new(Expression::BinExp(
                                Box::new(Expression::Value(Value::Number(2.into()))),
                                BinaryOperation::Mul,
                                Box::new(Expression::BinExp(
                                        Box::new(Expression::Value(Value::Number(2.into()))),
                                        BinaryOperation::Sub,
                                        Box::new(Expression::Value(Value::Number(4.into()))))
                                    ))
                                )
                            )
                        ),
                            BinaryOperation::Div,
                            Box::new(
                                Expression::Value(Value::Number(4.into()))
                            )
                        )
                    )
                ]
            },
        }
    }
}
