mod program;
mod validate;

use crate::token::{LexingError, Symbol, Token, TokenKind, TokenValue};
use crate::{FunctionArgument, FunctionDefinition};
use logos::Logos;
pub use program::{Element, Expression, Program, Statement};
use rigz_vm::{BinaryOperation, RigzType, UnaryOperation, Value};
use std::collections::VecDeque;
use std::fmt::{format, Debug};
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
        // let mut offset = 0;
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

    pub fn parse(&mut self) -> Result<Program<'lex>, LexingError> {
        let mut elements = Vec::new();
        while self.has_tokens() {
            elements.push(self.parse_element()?)
        }
        Ok(Program { elements })
    }
}

impl <'lex> From<TokenValue<'lex>> for Expression<'lex> {
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

    fn peek_required_token(&self) -> Result<&Token<'lex>, LexingError> {
        match self.tokens.front() {
            None => Err(Self::eoi_error()),
            Some(t) => Ok(t),
        }
    }

    fn next_token(&mut self) -> Option<Token<'lex>> {
        self.tokens.pop_front()
    }

    fn next_required_token(&mut self) -> Result<Token<'lex>, LexingError> {
        match self.next_token() {
            None => Err(Self::eoi_error()),
            Some(t) => Ok(t),
        }
    }

    fn consume_token(&mut self, kind: TokenKind<'lex>) -> Result<(), LexingError> {
        match self.next_token() {
            None => Err(Self::eoi_error()),
            Some(t) if t.kind != kind => Err(LexingError::ParseError(format!(
                "expected {}, received {:?}",
                kind, t
            ))),
            Some(_) => Ok(()),
        }
    }

    fn eoi_error() -> LexingError {
        LexingError::ParseError("Unexpected end of input".to_string())
    }

    fn parse_element(&mut self) -> Result<Element<'lex>, LexingError> {
        let token = match self.peek_token() {
            None => return Err(Self::eoi_error()),
            Some(t) => t
        };
        let ele = match token.kind {
            TokenKind::Let => {
                self.consume_token(TokenKind::Let)?;
                self.parse_assignment(false)?.into()
            },
            TokenKind::Mut => {
                self.consume_token(TokenKind::Mut)?;
                self.parse_assignment(true)?.into()
            },
            TokenKind::Identifier(id) => {
                self.consume_token(TokenKind::Identifier(id))?;
                match self.peek_token() {
                    None => id.into(),
                    Some(t) if t.kind == TokenKind::Assign =>
                        self.parse_assignment_definition(false, id)?.into(),
                    Some(_) => self.parse_identifier_expression(id)?.into(),
                }
            },
            TokenKind::FunctionDef => {
                self.parse_function_definition()?.into()
            }
            TokenKind::Newline => {
                self.consume_token(TokenKind::Newline)?;
                self.parse_element()?
            },
            _ => {
                self.parse_expression()?.into()
            },
        };
        Ok(ele)
    }

    fn parse_function_definition(&mut self) -> Result<Statement<'lex>, LexingError> {
        self.consume_token(TokenKind::FunctionDef)?;
        let next = self.next_required_token()?;
        if let TokenKind::Identifier(name) = next.kind {
            Ok(Statement::FunctionDefinition {
                name,
                type_definition: self.parse_function_type_definition()?,
                elements: self.parse_function_body()?,
            })
        } else {
            Err(LexingError::ParseError(format!(
                "Invalid Function definition {:?}",
                next
            )))
        }
    }

    fn parse_assignment(&mut self, mutable: bool) -> Result<Statement<'lex>, LexingError> {
        let next = self.next_required_token().map_err(|e| {
            LexingError::ParseError(format!("Expected token for assignment: {}", e))
        })?;

        if let TokenKind::Identifier(id) = next.kind {
            self.parse_assignment_definition(mutable, id)
        } else {
            Err(LexingError::ParseError(format!(
                "Unexpected token for assignment {:?}",
                next
            )))
        }
    }

    fn parse_assignment_definition(
        &mut self,
        mutable: bool,
        id: &'lex str,
    ) -> Result<Statement<'lex>, LexingError> {
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
    ) -> Result<Expression<'lex>, LexingError> {
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

    fn parse_paren_expression(&mut self) -> Result<Expression<'lex>, LexingError> {
        let expr = self.parse_expression()?;
        self.consume_token(TokenKind::Rparen)?;
        Ok(expr)
    }

    fn parse_inline_expression<LHS>(&mut self, lhs: LHS) -> Result<Expression<'lex>, LexingError>
    where
        LHS: Into<Expression<'lex>>,
    {
        let lhs = lhs.into();
        match self.next_token() {
            None => Ok(lhs),
            Some(t) => {
                // value, binary, .function_call
                match t.kind {
                    TokenKind::BinOp(op) => self.parse_inline_binary_expression(lhs, op),
                    TokenKind::Minus => {
                        let op = BinaryOperation::Sub;
                        self.parse_inline_binary_expression(lhs, op)
                    }
                    TokenKind::Pipe => {
                        let op = BinaryOperation::BitOr;
                        self.parse_inline_binary_expression(lhs, op)
                    }
                    TokenKind::Period => self.parse_instance_call(lhs),
                    TokenKind::Comma | TokenKind::Rparen | TokenKind::Rcurly | TokenKind::Assign => {
                        self.tokens.push_front(t);
                        Ok(lhs)
                    },
                    k => todo!("inline {}", k),
                }
            }
        }
    }

    fn parse_instance_call(
        &mut self,
        lhs: Expression<'lex>,
    ) -> Result<Expression<'lex>, LexingError> {
        // a.b.c.d 1, 2, 3
        todo!()
    }

    fn parse_value_expression(&mut self, value: TokenValue<'lex>) -> Result<Expression<'lex>, LexingError> {
        self.parse_inline_expression(value)
    }

    fn parse_symbol_expression(&mut self, symbol: Symbol<'lex>) -> Result<Expression<'lex>, LexingError> {
        self.parse_inline_expression(symbol)
    }

    fn parse_inline_binary_expression<LHS>(
        &mut self,
        lhs: LHS,
        op: BinaryOperation,
    ) -> Result<Expression<'lex>, LexingError>
    where
        LHS: Into<Expression<'lex>> + Debug,
    {
        let lhs = lhs.into();
        let next = self.next_required_token()?;
        match next.kind {
            TokenKind::Value(v) => self.parse_inner_binary_expression(lhs, op, v),
            TokenKind::Identifier(id) => self.parse_inner_binary_expression(lhs, op, id),
            TokenKind::Lparen => Ok(Expression::binary(lhs, op, self.parse_paren_expression()?)),
            t => todo!("{}", t),
        }
    }

    fn parse_inner_binary_expression<RHS>(
        &mut self,
        lhs: Expression<'lex>,
        op: BinaryOperation,
        rhs: RHS,
    ) -> Result<Expression<'lex>, LexingError>
    where
        RHS: Into<Expression<'lex>>,
    {
        let rhs = rhs.into();
        match self.peek_token() {
            None => Ok(Expression::binary(lhs, op, rhs)),
            Some(t) if t.terminal() => {
                self.consume_token(t.kind)?;
                Ok(Expression::binary(lhs, op, rhs))
            },
            Some(t) if t.kind == TokenKind::Rparen => {
                Ok(Expression::binary(lhs, op, rhs))
            },
            Some(t) => {
                if let TokenKind::BinOp(bop) = t.kind {
                    self.consume_token(TokenKind::BinOp(bop))?;
                    return Ok(Expression::BinExp(
                        Box::new(Expression::BinExp(Box::new(lhs), op, Box::new(rhs))),
                        bop,
                        Box::new(self.parse_expression()?),
                    ))
                }
                match t.kind {
                    TokenKind::Minus => {
                        self.consume_token(TokenKind::Minus)?;
                        Ok(Expression::BinExp(
                            Box::new(Expression::BinExp(Box::new(lhs), op, Box::new(rhs))),
                            BinaryOperation::Sub,
                            Box::new(self.parse_expression()?),
                        ))
                    }
                    TokenKind::Period => {
                        self.consume_token(TokenKind::Period)?;
                        Ok(Expression::binary(lhs, op, self.parse_instance_call(rhs)?))
                    }
                    TokenKind::Lparen => {
                        self.consume_token(TokenKind::Lparen)?;
                        Ok(Expression::binary(lhs, op, self.parse_paren_expression()?))
                    }
                    _ => {
                        Err(LexingError::ParseError(format!(
                            "Invalid Inner Expression {:?}",
                            t
                        )))
                    }
                }
            }
        }
    }

    fn parse_expression(&mut self) -> Result<Expression<'lex>, LexingError> {
        let next = self
            .next_required_token()
            .map_err(|e| LexingError::ParseError(format!("Invalid Expression {}", e)))?;
        match next.kind {
            TokenKind::Minus => self.parse_unary_expression(UnaryOperation::Neg),
            TokenKind::Not => self.parse_unary_expression(UnaryOperation::Not),
            TokenKind::Identifier(id) => self.parse_identifier_expression(id),
            TokenKind::Value(v) => self.parse_value_expression(v),
            TokenKind::Symbol(s) => self.parse_symbol_expression(s),
            TokenKind::Lparen => self.parse_paren_expression(),
            TokenKind::Lbracket => self.parse_list(),
            TokenKind::Lcurly => self.parse_map(),
            a => todo!("{}", a),
        }
    }

    fn parse_unary_expression(
        &mut self,
        op: UnaryOperation,
    ) -> Result<Expression<'lex>, LexingError> {
        let exp = self.parse_expression()?;
        Ok(Expression::UnaryExp(op, Box::new(exp)))
    }

    fn parse_args(&mut self) -> Result<Vec<Expression<'lex>>, LexingError> {
        let mut args = Vec::new();
        loop {
            match self.peek_token() {
                None => break,
                Some(t) if t.terminal() => {
                    self.consume_token(t.kind.clone())?;
                    break
                },
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

    fn parse_list(&mut self) -> Result<Expression<'lex>, LexingError> {
        let mut args = Vec::new();
        loop {
            match self.peek_token() {
                None => return Err(LexingError::ParseError("Missing ]".to_string())),
                Some(t) if t.kind == TokenKind::Rbracket => {
                    self.consume_token(TokenKind::Rbracket)?;
                    break
                },
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

    fn parse_map(&mut self) -> Result<Expression<'lex>, LexingError> {
        let mut args = Vec::new();

        loop {
            match self.peek_token() {
                None => return Err(LexingError::ParseError("Missing }".to_string())),
                Some(t) if t.kind == TokenKind::Rcurly => {
                    self.consume_token(TokenKind::Rcurly)?;
                    break
                },
                Some(t) if t.kind == TokenKind::Comma => {
                    self.consume_token(TokenKind::Comma)?;
                    break
                },
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

    fn parse_function_arguments(&mut self) -> Result<Vec<FunctionArgument<'lex>>, LexingError> {
        let mut args = Vec::new();
        let next = self.peek_required_token()?;
        if next.kind != TokenKind::Lparen {
            return Ok(args)
        }

        self.consume_token(TokenKind::Lparen)?;

        loop {
            match self.peek_token() {
                None => break,
                Some(t) if t.kind == TokenKind::Rparen => {
                    self.consume_token(TokenKind::Rparen)?;
                    break
                },
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

    fn parse_function_argument(&mut self) -> Result<FunctionArgument<'lex>, LexingError> {
        let next = self.next_required_token()?;
        if let TokenKind::Identifier(name) = next.kind {
            let rigz_type = match self.peek_required_token()?.kind.clone() {
                TokenKind::Colon => {
                    self.consume_token(TokenKind::Colon)?;
                    self.parse_return_type()?
                }
                _ => RigzType::Any
            };
            Ok(FunctionArgument {
                name: Some(name),
                default: None,
                rigz_type,
            })
        } else {
            Err(LexingError::ParseError(format!("Invalid Function Argument {:?}", next)))
        }
    }

    fn parse_return_type(&mut self) -> Result<RigzType, LexingError> {
        let mut rigz_type = RigzType::Any;
        match self.peek_token() {
            None => return Err(Self::eoi_error()),
            Some(t) => {
                if t.kind == TokenKind::Arrow {
                    self.consume_token(TokenKind::Arrow)?;
                    rigz_type = self.parse_rigz_type()?
                }
            }
        }
        Ok(rigz_type)
    }

    fn parse_rigz_type(&mut self) -> Result<RigzType, LexingError> {
        let next = self.next_required_token()?;
        if let TokenKind::TypeValue(id) = next.kind {
            let t = match id {
                "Any" => RigzType::Any,
                "Number" => RigzType::Number,
                "String" => RigzType::String,
                "None" => RigzType::None,
                "Error" => RigzType::Error,
                "List" => RigzType::List,
                "Map" => RigzType::Map,
                "Bool" => RigzType::Bool,
                _ => todo!()
            };
            Ok(t)
        } else {
            Err(LexingError::ParseError(format!("Invalid type {:?}", next)))
        }
    }

    fn parse_function_type_definition(&mut self) -> Result<FunctionDefinition<'lex>, LexingError> {
        Ok(FunctionDefinition {
            arguments: self.parse_function_arguments()?,
            return_type: self.parse_return_type()?,
            positional: true,
        })
    }

    fn parse_function_body(&mut self) -> Result<Vec<Element<'lex>>, LexingError> {
        let mut body = vec![];
        loop {
            let next = self.peek_required_token()?;
            match next.kind.clone() {
                TokenKind::End => break,
                _ => body.push(self.parse_element()?)
            }
        }
        Ok(body)
    }
}

// TODO better error messages
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
                    let v = parse(input);
                    assert_eq!(v, Ok($expected), "Failed to parse input: {}", input)
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
                    let v = parse(input).err();
                    assert_eq!(v, Some($expected), "Successfully parsed invalid input: {}", input)
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
                    assert_eq!(v.is_err(), true, "Failed to parse {}", input);
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

        // todo support later
        // test_parse! {
        //     multi_complex_parens "1 + (2 * (2 - 4)) / 4" = Program {
        //         elements: vec![
        //             Element::Expression(
        //                 Expression::BinExp(
        //                     Box::new(Expression::BinExp(
        //                     Box::new(Expression::Value(Value::Number(1.into()))),
        //                     BinaryOperation::Add,
        //                     Box::new(Expression::BinExp(
        //                         Box::new(Expression::Value(Value::Number(2.into()))),
        //                         BinaryOperation::Mul,
        //                         Box::new(Expression::BinExp(
        //                                 Box::new(Expression::Value(Value::Number(2.into()))),
        //                                 BinaryOperation::Sub,
        //                                 Box::new(Expression::Value(Value::Number(4.into()))))
        //                             ))
        //                         )
        //                     )
        //                 ),
        //                     BinaryOperation::Div,
        //                     Box::new(
        //                         Expression::Value(Value::Number(4.into()))
        //                     )
        //                 )
        //             )
        //         ]
        //     },
        // }
    }
}
