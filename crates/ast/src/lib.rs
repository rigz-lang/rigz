mod modules;
mod program;
mod token;
mod validate;

#[cfg(feature = "derive")]
mod ast_derive;

#[cfg(feature = "format")]
mod format;

#[cfg(feature = "format")]
pub use format::format;

use logos::Logos;
pub use modules::{ParsedDependency, ParsedModule, ParsedObject};
pub use program::*;

use rigz_core::*;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::path::PathBuf;
pub use token::ParsingError;
use token::{Symbol, Token, TokenKind, TokenValue};
pub use validate::*;

#[derive(Default, Debug, Clone)]
pub struct ParserOptions {
    current_directory: Option<PathBuf>,
    debug: bool,
    disable_file_imports: bool,
    disable_url_imports: bool,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Parser<'t> {
    pub input: Option<String>,
    tokens: VecDeque<Token<'t>>,
    line: usize, // todo repl should set this
}

// TODO better error messages
pub fn parse(input: &str, debug: bool) -> Result<Program, ParsingError> {
    Parser::prepare(input, debug)?.parse()
}

impl<'t> Parser<'t> {
    pub fn prepare(input: &'t str, debug: bool) -> Result<Self, ParsingError> {
        let input = input.trim(); // ensure no trailing newlines to avoid issues in parse_element
        if input.is_empty() {
            return Err(ParsingError::ParseError(
                "Invalid Input, no tokens".to_string(),
            ));
        }

        let mut lexer = TokenKind::lexer(input);
        let mut tokens = VecDeque::new();
        let mut line = 1;
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
        let input = if debug { Some(input.to_string()) } else { None };
        Ok(Parser {
            input,
            tokens,
            line,
        })
    }

    pub fn parse(mut self) -> Result<Program, ParsingError> {
        let mut elements = Vec::new();
        while self.has_tokens() {
            elements.push(self.parse_element()?)
        }
        Ok(Program {
            input: self.input,
            elements,
        })
    }

    pub fn parse_module_trait_definition(&mut self) -> Result<ModuleTraitDefinition, ParsingError> {
        let mut next = self.next_required_token("parse_module_trait_definition")?;
        let auto_import = if next.kind == TokenKind::Import {
            next = self.next_required_token("parse_module_trait_definition")?;
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

impl<'t> From<TokenValue<'t>> for Expression {
    #[inline]
    fn from(value: TokenValue<'t>) -> Self {
        Expression::Value(value.into())
    }
}

impl From<&str> for Expression {
    #[inline]
    fn from(value: &str) -> Self {
        Expression::Identifier(value.to_string())
    }
}

impl From<Symbol<'_>> for Expression {
    #[inline]
    fn from(value: Symbol) -> Self {
        Expression::Symbol(value.0.to_string())
    }
}

impl<T: Into<Expression>> From<T> for Element {
    #[inline]
    fn from(value: T) -> Self {
        Element::Expression(value.into())
    }
}

impl From<Statement> for Element {
    #[inline]
    fn from(value: Statement) -> Self {
        Element::Statement(value)
    }
}

impl<'t> Parser<'t> {
    fn peek_token(&self) -> Option<Token<'t>> {
        self.tokens.front().cloned()
    }

    pub fn has_tokens(&self) -> bool {
        !self.tokens.is_empty()
    }

    fn peek_required_token(&self, location: &'static str) -> Result<Token<'t>, ParsingError> {
        match self.peek_token() {
            None => Err(Self::eoi_error("peek_required_token", location)),
            Some(t) => Ok(t),
        }
    }

    fn peek_required_token_eat_newlines(
        &mut self,
        location: &'static str,
    ) -> Result<Token<'t>, ParsingError> {
        match self.peek_token() {
            None => Err(Self::eoi_error("peek_required_token", location)),
            Some(t) if t.kind == TokenKind::Newline => {
                self.consume_token(TokenKind::Newline)?;
                self.peek_required_token_eat_newlines(location)
            }
            Some(t) => Ok(t),
        }
    }

    fn next_token(&mut self) -> Option<Token<'t>> {
        self.tokens.pop_front()
    }

    fn next_required_token(&mut self, caller: &'static str) -> Result<Token<'t>, ParsingError> {
        match self.next_token() {
            None => Err(Self::eoi_error("next_required_token", caller)),
            Some(t) => Ok(t),
        }
    }

    fn consume_token(&mut self, kind: TokenKind<'t>) -> Result<(), ParsingError> {
        match self.next_token() {
            None => Err(Self::eoi_error_string(format!("expected {}", kind))),
            Some(t) if t.kind != kind => Err(ParsingError::ParseError(format!(
                "expected {}, received {:?}",
                kind, t
            ))),
            Some(_) => Ok(()),
        }
    }

    fn consume_token_eat_newlines(&mut self, kind: TokenKind<'t>) -> Result<(), ParsingError> {
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

    fn eoi_error(location: &'static str, caller: &'static str) -> ParsingError {
        ParsingError::Eoi(format!("{location} - {caller}"))
    }

    fn eoi_error_string(message: String) -> ParsingError {
        ParsingError::Eoi(message)
    }

    fn parse_element(&mut self) -> Result<Element, ParsingError> {
        let token = match self.peek_token() {
            None => return Err(Self::eoi_error_string("parse_element".to_string())),
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
            TokenKind::Impl => {
                self.consume_token(TokenKind::Impl)?;
                let base_trait = self.parse_rigz_type(None, false)?;
                self.consume_token(TokenKind::For)?;
                let concrete = self.parse_rigz_type(None, false)?;
                let mut definitions = Vec::new();
                loop {
                    let t = self.peek_required_token_eat_newlines("parse_element")?;
                    if t.kind == TokenKind::End {
                        self.consume_token(TokenKind::End)?;
                        break;
                    }
                    self.consume_token(TokenKind::FunctionDef)?;
                    definitions.push(self.parse_function_definition(None)?);
                }
                Statement::TraitImpl {
                    base_trait,
                    concrete,
                    definitions,
                }
                .into()
            }
            TokenKind::Lparen => {
                self.consume_token(TokenKind::Lparen)?;
                let e = self.parse_paren_expression()?;
                if let Element::Expression(e) = e {
                    self.parse_expression_suffix(e)?.into()
                } else {
                    e
                }
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
                                lhs: Assign::Identifier(id.to_string(), false),
                                op: BinaryOperation::Add,
                                expression: Expression::Value(1.into()),
                            }
                            .into()
                        }
                        TokenKind::Decrement => {
                            self.consume_token(TokenKind::Decrement)?;
                            Statement::BinaryAssignment {
                                lhs: Assign::Identifier(id.to_string(), false),
                                op: BinaryOperation::Sub,
                                expression: Expression::Value(1.into()),
                            }
                            .into()
                        }
                        TokenKind::BinAssign(op) => {
                            self.consume_token(TokenKind::BinAssign(op))?;
                            Statement::BinaryAssignment {
                                lhs: Assign::Identifier(id.to_string(), false),
                                op,
                                expression: self.parse_expression()?,
                            }
                            .into()
                        }
                        _ => self.parse_identifier_element(id)?.into(),
                    },
                }
            }
            TokenKind::Type => {
                self.consume_token(TokenKind::Type)?;
                let next = self.next_required_token("parse_element - TypeDefinition")?;
                if let TokenKind::TypeValue(name) = next.kind {
                    self.consume_token(TokenKind::Assign)?;
                    Statement::TypeDefinition(
                        name.to_string(),
                        self.parse_rigz_type(Some(name), false)?,
                    )
                    .into()
                } else {
                    return Err(ParsingError::ParseError(format!(
                        "Invalid type definition expected TypeValue, received {:?}",
                        next
                    )));
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
                        _ => self.parse_this_element()?,
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
            TokenKind::Lifecycle(lifecycle) => self.parse_lifecycle_func(lifecycle)?.into(),
            TokenKind::Object => {
                Statement::ObjectDefinition(self.parse_object_definition()?).into()
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

        self.parse_element_suffix(ele)
    }

    fn parse_element_suffix(&mut self, element: Element) -> Result<Element, ParsingError> {
        let next = self.peek_token();
        match next {
            None => Ok(element),
            Some(t) => {
                let (exp, assn) = match element {
                    Element::Statement(s) => match s {
                        Statement::Assignment { lhs, expression } => {
                            (expression, Some((lhs, None)))
                        }
                        Statement::BinaryAssignment {
                            lhs,
                            op,
                            expression,
                        } => (expression, Some((lhs, Some(op)))),
                        s => return Ok(Element::Statement(s)),
                    },
                    Element::Expression(e) => (e, None),
                };
                match t.kind {
                    TokenKind::Newline | TokenKind::Into => {
                        let mut new_lines = 0;
                        for t in &self.tokens {
                            match t.kind {
                                TokenKind::Into => break,
                                TokenKind::Newline => {
                                    new_lines += 1;
                                }
                                _ => {
                                    let el = match assn {
                                        None => Element::Expression(exp),
                                        Some((assn, op)) => match op {
                                            None => Statement::Assignment {
                                                lhs: assn,
                                                expression: exp,
                                            }
                                            .into(),
                                            Some(op) => Statement::BinaryAssignment {
                                                lhs: assn,
                                                op,
                                                expression: exp,
                                            }
                                            .into(),
                                        },
                                    };
                                    return Ok(el);
                                }
                            }
                        }
                        for _ in 0..new_lines {
                            self.consume_token(TokenKind::Newline)?;
                        }
                        self.consume_token(TokenKind::Into)?;
                        let fe = self.parse_expression()?;

                        let fe = match fe {
                            Expression::Function(fe) => fe,
                            Expression::Identifier(id) => FunctionExpression::FunctionCall(
                                id,
                                RigzArguments::Positional(vec![]),
                            ),
                            _ => {
                                return Err(ParsingError::ParseError(format!(
                                    "Invalid expression after {t:?}, {fe:?} a Function is required"
                                )));
                            }
                        };
                        let mut exp = Expression::Into {
                            base: exp.into(),
                            next: fe,
                        };

                        loop {
                            let t = self.peek_token();
                            match t {
                                None => break,
                                Some(t) if t.kind == TokenKind::Newline => {
                                    self.consume_token(TokenKind::Newline)?;
                                }
                                Some(t) if t.kind == TokenKind::Into => {
                                    self.consume_token(TokenKind::Into)?;
                                    let fe = self.parse_expression()?;
                                    let Expression::Function(fe) = fe else {
                                        return Err(ParsingError::ParseError(format!("Invalid expression after {t:?}, {fe:?} a Function call is required")));
                                    };

                                    exp = Expression::Into {
                                        base: exp.into(),
                                        next: fe,
                                    };
                                }
                                Some(_) => break,
                            }
                        }
                        Ok(exp.into())
                    }
                    _ => {
                        let el = match assn {
                            None => Element::Expression(exp),
                            Some((assn, op)) => match op {
                                None => Statement::Assignment {
                                    lhs: assn,
                                    expression: exp,
                                }
                                .into(),
                                Some(op) => Statement::BinaryAssignment {
                                    lhs: assn,
                                    op,
                                    expression: exp,
                                }
                                .into(),
                            },
                        };
                        Ok(el)
                    }
                }
            }
        }
    }

    fn parse_lifecycle(&mut self, lifecycle: &'t str) -> Result<Lifecycle, ParsingError> {
        self.consume_token(TokenKind::Lifecycle(lifecycle))?;
        match lifecycle {
            // todo support @test.assert_eq, @test.assert_neq, @test.assert
            "test" => Ok(Lifecycle::Test(TestLifecycle)),
            "memo" => Ok(Lifecycle::Memo(MemoizedLifecycle::default())),
            "on" => {
                self.consume_token(TokenKind::Lparen)?;
                let e = self.parse_paren_expression()?;
                match e {
                    Element::Expression(Expression::Value(PrimitiveValue::String(s))) => {
                        Ok(Lifecycle::On(EventLifecycle { event: s }))
                    }
                    _ => Err(ParsingError::ParseError(format!(
                        "Expressions not supported for `on` lifecycle {e:?}"
                    ))),
                }
            }
            _ => Err(ParsingError::ParseError(format!(
                "Lifecycle {lifecycle} is not supported"
            ))),
        }
    }

    fn parse_lifecycle_func(
        &mut self,
        initial_lifecycle: &'t str,
    ) -> Result<Statement, ParsingError> {
        let mut lifecycle = self.parse_lifecycle(initial_lifecycle)?;
        loop {
            let next = self.peek_required_token_eat_newlines("parse_lifecycle_func")?;
            if let TokenKind::Lifecycle(t) = next.kind {
                match &mut lifecycle {
                    Lifecycle::Composite(v) => {
                        v.push(self.parse_lifecycle(t)?);
                    }
                    l => {
                        *l = Lifecycle::Composite(vec![l.clone(), self.parse_lifecycle(t)?]);
                    }
                }
            } else {
                break;
            }
        }
        self.consume_token_eat_newlines(TokenKind::FunctionDef)?;
        Ok(Statement::FunctionDefinition(
            self.parse_function_definition(Some(lifecycle))?,
        ))
    }

    fn parse_import(&mut self) -> Result<Statement, ParsingError> {
        self.consume_token(TokenKind::Import)?;
        let next = self.next_required_token("parse_import")?;
        let import_value = match next.kind {
            TokenKind::TypeValue(tv) => {
                ImportValue::TypeValue(tv.to_string())
            }
            TokenKind::Value(TokenValue::String(s)) => {
                if s.starts_with("http") {
                    ImportValue::UrlPath(s.to_string())
                } else {
                    ImportValue::FilePath(s.to_string())
                }
            }
            t => return Err(ParsingError::ParseError(format!(
                "Only type values and string literals are supported in import currently, received {t}"
            ))),
        };
        Ok(Statement::Import(import_value))
    }

    fn parse_expression(&mut self) -> Result<Expression, ParsingError> {
        let next = self
            .next_required_token("parse_expression")
            .map_err(|e| ParsingError::ParseError(format!("Invalid Expression {e}")))?;
        let exp = match next.kind {
            TokenKind::Minus => self.parse_unary_expression(UnaryOperation::Neg)?,
            TokenKind::Not => self.parse_unary_expression(UnaryOperation::Not)?,
            TokenKind::Identifier(id) => self.parse_identifier_expression(id)?,
            TokenKind::Value(v) => self.parse_value_expression(v)?,
            TokenKind::This => self.parse_this_expression()?,
            TokenKind::Symbol(s) => self.parse_symbol_expression(s)?,
            TokenKind::Lparen => {
                let paren = self.parse_paren_expression()?;
                let Element::Expression(e) = paren else {
                    return Err(ParsingError::ParseError(format!(
                        "Element found instead of expression {paren:?}"
                    )));
                };
                e
            }
            TokenKind::Lbracket => self.parse_list()?,
            TokenKind::Lcurly => self.parse_map()?,
            TokenKind::Do => {
                let next = self.peek_required_token("parse_expression - do")?;
                match next.kind {
                    TokenKind::Pipe => {
                        self.consume_token(next.kind)?;
                        let (arguments, var_args_start) = self.parse_lambda_arguments()?;
                        Expression::Lambda {
                            arguments,
                            var_args_start,
                            body: Box::new(Expression::Scope(self.parse_scope()?)),
                        }
                    }
                    TokenKind::BinOp(BinaryOperation::Or) => {
                        self.consume_token(next.kind)?;
                        Expression::Lambda {
                            arguments: vec![],
                            var_args_start: None,
                            body: Box::new(Expression::Scope(self.parse_scope()?)),
                        }
                    }
                    _ => Expression::Scope(self.parse_scope()?),
                }
            }
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
                let next = self.peek_token();
                match next {
                    None => Expression::Value(PrimitiveValue::Type(type_value)),
                    Some(t) if t.kind == TokenKind::Period => {
                        self.consume_token(TokenKind::Period)?;
                        let func_name =
                            self.next_required_token("parse_expression - TypeFunctionCall")?;
                        match func_name.kind {
                            TokenKind::Identifier(func_name) => {
                                let (args, assign) = self.parse_args()?;
                                if assign {
                                    let t = self.next_required_token("parse_expression: =")?;
                                    return Err(ParsingError::ParseError(format!(
                                        "Unexpected = after {args:?} - {t:?}"
                                    )));
                                }
                                FunctionExpression::TypeFunctionCall(
                                    type_value,
                                    func_name.to_string(),
                                    args,
                                )
                                .into()
                            }
                            TokenKind::New => {
                                let (args, assign) = self.parse_args()?;
                                if assign {
                                    let t = self.next_required_token("parse_expression: =")?;
                                    return Err(ParsingError::ParseError(format!(
                                        "Unexpected = after {args:?} - {t:?}"
                                    )));
                                }
                                FunctionExpression::TypeConstructor(type_value, args).into()
                            }
                            _ => {
                                return Err(ParsingError::ParseError(format!(
                                    "Invalid Token for Type Function Call {:?}",
                                    func_name
                                )));
                            }
                        }
                    }
                    Some(_) => Expression::Value(PrimitiveValue::Type(type_value)),
                }
            }
            TokenKind::Error => {
                let ex = self.parse_expression()?;
                Expression::Error(ex.into())
            }
            TokenKind::Return => match self.peek_token() {
                None => Expression::Return(None),
                Some(t) => {
                    if t.terminal() {
                        self.consume_token(t.kind)?;
                        Expression::Return(None)
                    } else {
                        Expression::Return(Some(Box::new(self.parse_expression()?)))
                    }
                }
            },
            TokenKind::Pipe => self.parse_lambda(false)?,
            TokenKind::BinOp(BinaryOperation::Or) => self.parse_lambda(true)?,
            _ => {
                return Err(ParsingError::ParseError(format!(
                    "Invalid Token for Expression {:?}",
                    next
                )))
            }
        };
        self.parse_expression_suffix(exp)
    }

    fn parse_lambda(&mut self, empty: bool) -> Result<Expression, ParsingError> {
        let (arguments, var_args_start) = if empty {
            (vec![], None)
        } else {
            self.parse_lambda_arguments()?
        };
        let body = self.parse_expression()?;
        Ok(Expression::Lambda {
            arguments,
            var_args_start,
            body: Box::new(body),
        })
    }

    fn parse_expression_suffix(&mut self, exp: Expression) -> Result<Expression, ParsingError> {
        // todo should other suffix tokens be allowed to have leading newlines?
        match self.peek_token() {
            None => Ok(exp),
            Some(t) if t.kind == TokenKind::Semi => {
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
                    Ok(Expression::Cast(
                        Box::new(exp),
                        self.parse_rigz_type(None, false)?,
                    ))
                }
                TokenKind::BinOp(_) | TokenKind::Pipe | TokenKind::Minus | TokenKind::Period => {
                    Ok(self.parse_inline_expression(exp)?)
                }
                TokenKind::Lbracket => {
                    self.consume_token(TokenKind::Lbracket)?;
                    let index = self.parse_expression()?;
                    self.consume_token(TokenKind::Rbracket)?;
                    let mut base = Expression::Index(exp.into(), index.into());
                    loop {
                        let next = self.peek_token();
                        match next {
                            Some(t) if t.kind == TokenKind::Lbracket => {
                                self.consume_token(TokenKind::Lbracket)?;
                                let index = self.parse_expression()?;
                                self.consume_token(TokenKind::Rbracket)?;
                                // todo handle edge case where indexed value may be a function that accepts a list
                                // v = { a = |list| list + [1, 2, 3] }; key = 'a'; v[key] [4, 5, 6]
                                base = Expression::Index(base.into(), index.into())
                            }
                            _ => break,
                        }
                    }
                    Ok(base)
                }
                _ => Ok(exp),
            },
        }
    }

    fn parse_assignment(&mut self, mutable: bool) -> Result<Statement, ParsingError> {
        let next = self
            .next_required_token("parse_assignment")
            .map_err(|e| ParsingError::ParseError(format!("Expected token for assignment: {e}")))?;

        match next.kind {
            TokenKind::Identifier(id) => self.parse_assignment_definition(mutable, id),
            TokenKind::Lparen => self.parse_tuple_assign(mutable),
            _ => Err(ParsingError::ParseError(format!(
                "Unexpected token for assignment {:?}",
                next
            ))),
        }
    }

    fn parse_tuple_assign(&mut self, mutable: bool) -> Result<Statement, ParsingError> {
        let mut tuple = vec![];
        let mut is_mut = mutable;
        let mut needs_id = false;
        loop {
            let next = self.next_required_token("parse_tuple_assign")?;
            match next.kind {
                TokenKind::Rparen => {
                    break;
                }
                TokenKind::Comma => {
                    if needs_id {
                        return Err(ParsingError::ParseError(format!(
                            "missing identifier after {}",
                            if is_mut { "mut" } else { "let" }
                        )));
                    }
                    continue;
                }
                TokenKind::Let => {
                    is_mut = false;
                    needs_id = true;
                }
                TokenKind::Mut => {
                    is_mut = true;
                    needs_id = true;
                }
                TokenKind::Identifier(id) => {
                    tuple.push((id.to_string(), is_mut));
                    is_mut = mutable;
                    needs_id = false
                }
                _ => {
                    return Err(ParsingError::ParseError(format!(
                        "Unexpected token in tuple assign {next:?}"
                    )))
                }
            }
        }
        self.consume_token(TokenKind::Assign)?;
        Ok(Statement::Assignment {
            lhs: Assign::Tuple(tuple),
            expression: self.parse_expression()?,
        })
    }

    fn parse_assignment_definition(
        &mut self,
        mutable: bool,
        id: &'t str,
    ) -> Result<Statement, ParsingError> {
        let token = self.peek_required_token("parse_assignment_definition")?;
        let rigz_type = match token.kind {
            TokenKind::Colon => {
                self.consume_token(TokenKind::Colon)?;
                Some(self.parse_rigz_type(None, false)?)
            }
            _ => None,
        };
        self.consume_token(TokenKind::Assign)?;
        let lhs = match rigz_type {
            None => Assign::Identifier(id.to_string(), mutable),
            Some(rigz_type) => Assign::TypedIdentifier(id.to_string(), mutable, rigz_type),
        };
        Ok(Statement::Assignment {
            lhs,
            expression: self.parse_expression()?,
        })
    }

    fn parse_this_assignment_definition(&mut self) -> Result<Statement, ParsingError> {
        self.consume_token(TokenKind::Assign)?;
        Ok(Statement::Assignment {
            lhs: Assign::This,
            expression: self.parse_expression()?,
        })
    }

    fn parse_identifier_element(&mut self, id: &'t str) -> Result<Element, ParsingError> {
        let args = match self.peek_token() {
            None => return Ok(id.into()),
            Some(next) => match next.kind {
                TokenKind::Value(_)
                | TokenKind::Identifier(_)
                | TokenKind::Symbol(_)
                | TokenKind::Lparen
                | TokenKind::Lcurly
                | TokenKind::This
                | TokenKind::Lbracket
                // if/unless not allowed as args without parens
                | TokenKind::Do => {
                    let (args, assign) = self.parse_args()?;
                    if assign {
                        let t = self.next_required_token("parse_identifier_element - =")?;
                        return Err(ParsingError::ParseError(format!("Unexpected = after {args:?} - {t:?}")))
                    }
                    args
                },
                _ => return self.parse_inline_element(id),
            },
        };
        Ok(FunctionExpression::FunctionCall(id.to_string(), args).into())
    }

    fn parse_identifier_expression(&mut self, id: &'t str) -> Result<Expression, ParsingError> {
        let args = match self.peek_token() {
            None => return Ok(id.into()),
            Some(next) => match next.kind {
                TokenKind::Value(_)
                | TokenKind::Identifier(_)
                | TokenKind::Symbol(_)
                | TokenKind::Lparen
                | TokenKind::Lcurly
                | TokenKind::This
                | TokenKind::Lbracket
                // if/unless not allowed as args without parens
                | TokenKind::Do => {
                    let (args, assign) = self.parse_args()?;
                    if assign {
                        let t = self.next_required_token("parse_identifier_expression - =")?;
                        return Err(ParsingError::ParseError(format!("Unexpected = after {args:?} - {t:?}")))
                    }
                    args
                },
                _ => return self.parse_inline_expression(id),
            },
        };
        Ok(FunctionExpression::FunctionCall(id.to_string(), args).into())
    }

    fn parse_identifier_expression_skip_inline(
        &mut self,
        id: &'t str,
    ) -> Result<Expression, ParsingError> {
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
                | TokenKind::Lbracket
                | TokenKind::Do => {
                    let (args, assign) = self.parse_args()?;
                    if assign {
                        let t = self
                            .next_required_token("parse_identifier_expression_skip_inline - =")?;
                        return Err(ParsingError::ParseError(format!(
                            "Unexpected = after {args:?} - {t:?}"
                        )));
                    }
                    args
                }
                _ => return Ok(id.into()),
            },
        };
        Ok(FunctionExpression::FunctionCall(id.to_string(), args).into())
    }

    fn parse_paren_expression(&mut self) -> Result<Element, ParsingError> {
        let mut expr = self.parse_expression()?;
        let t = self.next_required_token("parse_paren_expression")?;
        match t.kind {
            TokenKind::Rparen => {}
            TokenKind::Comma => {
                let t = self.parse_tuple(expr)?;
                let Element::Expression(e) = t else {
                    return Ok(t);
                };
                expr = e;
            }
            _ => {
                return Err(ParsingError::ParseError(format!(
                    "Invalid paren expression {t:?}"
                )))
            }
        }
        match self.peek_token() {
            None => Ok(expr.into()),
            Some(t) => match t.kind {
                TokenKind::Period => {
                    self.consume_token(TokenKind::Period)?;
                    Ok(self.parse_instance_call(expr)?.into())
                }
                _ => Ok(expr.into()),
            },
        }
    }

    fn parse_tuple(&mut self, first: Expression) -> Result<Element, ParsingError> {
        let mut tuple = vec![first];
        let mut assign = vec![];
        let mut is_assign = false;
        let mut is_mut = false;
        let mut needs_id = false;
        loop {
            let next = self.peek_required_token("parse_tuple")?;
            match next.kind {
                TokenKind::Rparen => {
                    self.consume_token(TokenKind::Rparen)?;
                    break;
                }
                TokenKind::Comma => {
                    if needs_id {
                        return Err(ParsingError::ParseError(format!(
                            "missing identifier after {}",
                            if is_mut { "mut" } else { "let" }
                        )));
                    }
                    self.consume_token(TokenKind::Comma)?;
                }
                TokenKind::Mut => {
                    is_assign = true;
                    is_mut = true;
                    needs_id = true;
                }
                TokenKind::Let => {
                    is_assign = true;
                    is_mut = false;
                    needs_id = true;
                }
                _ if !is_assign => tuple.push(self.parse_expression()?),
                TokenKind::Identifier(id) => {
                    assign = convert_to_assign(&mut tuple)?;
                    needs_id = false;
                    assign.push((id.to_string(), is_mut));
                    is_mut = false;
                }
                _ => {
                    return Err(ParsingError::ParseError(format!(
                        "Invalid tuple assign {next:?}"
                    )))
                }
            }
        }
        match self.peek_token() {
            None if !is_assign => Ok(Expression::Tuple(tuple).into()),
            Some(t) if t.kind == TokenKind::Assign => {
                self.consume_token(TokenKind::Assign)?;
                let assign = if tuple.is_empty() {
                    assign
                } else {
                    convert_to_assign(&mut tuple)?
                };
                Ok(Element::Statement(Statement::Assignment {
                    lhs: Assign::Tuple(assign),
                    expression: self.parse_expression()?,
                }))
            }
            Some(_) if !is_assign => Ok(Expression::Tuple(tuple).into()),
            _ => Err(ParsingError::ParseError(
                "Missing required = for tuple assign".to_string(),
            )),
        }
    }

    fn parse_inline_element<LHS>(&mut self, lhs: LHS) -> Result<Element, ParsingError>
    where
        LHS: Into<Expression>,
    {
        let mut res = lhs.into();
        if matches!(res, Expression::Lambda { .. }) {
            return Ok(res.into());
        }

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
                        match self.parse_instance_call_element(res)? {
                            Element::Expression(e) => {
                                res = e;
                            }
                            el => return Ok(el)
                        }
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
                    | TokenKind::Assign // for maps
                    | TokenKind::Colon // named args
                    | TokenKind::End => {
                        self.tokens.push_front(next);
                        break;
                    }
                    TokenKind::If | TokenKind::Unless => {
                        self.tokens.push_front(next);
                        res = self.parse_expression_suffix(res)?;
                    }
                    _ => return Err(ParsingError::ParseError(format!("Unexpected {:?} for inline expression", next)))
                },
            }
        }
        Ok(res.into())
    }

    fn parse_inline_expression<LHS>(&mut self, lhs: LHS) -> Result<Expression, ParsingError>
    where
        LHS: Into<Expression>,
    {
        match self.parse_inline_element(lhs)? {
            Element::Statement(s) => Err(ParsingError::ParseError(format!(
                "Unexpected statement for inline expression {s:?}"
            ))),
            Element::Expression(e) => Ok(e),
        }
    }

    fn parse_binary_expression(
        &mut self,
        lhs: Expression,
        op: BinaryOperation,
    ) -> Result<Expression, ParsingError> {
        let next = self.next_required_token("parse_binary_expression")?;
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
            TokenKind::Lparen => {
                let e = self.parse_paren_expression()?;
                let Element::Expression(e) = e else {
                    return Err(ParsingError::ParseError(format!(
                        "Elements not supported in binary expression {e:?}"
                    )));
                };
                e
            }
            TokenKind::Lcurly => self.parse_map()?,
            TokenKind::Lbracket => self.parse_list()?,
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

    fn parse_instance_call(&mut self, lhs: Expression) -> Result<Expression, ParsingError> {
        match self.parse_instance_call_element(lhs)? {
            Element::Statement(s) => Err(ParsingError::ParseError(format!(
                "Unexpected statement in place of expression, {s:?}"
            ))),
            Element::Expression(e) => Ok(e),
        }
    }

    fn parse_instance_call_element(&mut self, lhs: Expression) -> Result<Element, ParsingError> {
        let next = self.next_required_token("parse_instance_call_element")?;
        let mut lhs = lhs;
        let mut calls = match next.kind {
            TokenKind::Identifier(id) => {
                vec![id.to_string()]
            }
            TokenKind::Value(TokenValue::Number(Number::Int(n))) => {
                lhs = Expression::Index(lhs.into(), Expression::Value(n.into()).into());
                vec![]
            }
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
                        match t.kind {
                            TokenKind::Period => {
                                self.consume_token(TokenKind::Period)?;
                                needs_separator = false;
                                continue;
                            }
                            // todo how to handle this
                            // TokenKind::Lbracket => {
                            //     // a.b.c [1, 2, 3]
                            //     // a.b.c[1]
                            // }
                            _ => {
                                break;
                            }
                        }
                    } else {
                        match t.kind {
                            TokenKind::Identifier(n) => {
                                self.consume_token(TokenKind::Identifier(n))?;
                                calls.push(n.to_string());
                                needs_separator = true;
                                continue;
                            }
                            TokenKind::Value(TokenValue::Number(Number::Int(n))) => {
                                let base = if !calls.is_empty() {
                                    FunctionExpression::InstanceFunctionCall(
                                        Box::new(lhs),
                                        calls,
                                        vec![].into(),
                                    )
                                    .into()
                                } else {
                                    lhs.into()
                                };
                                lhs = Expression::Index(base, Expression::Value(n.into()).into());
                                calls = vec![];
                                needs_separator = true;
                            }
                            _ => {
                                return Err(ParsingError::ParseError(format!(
                                    "Unexpected {:?} for instance call, {:?}.{}",
                                    t,
                                    lhs,
                                    calls.join(".")
                                )))
                            }
                        }
                    }
                }
            }
        }

        if !calls.is_empty() {
            let (args, assign) = self.parse_args()?;
            if assign {
                return if args.is_empty() {
                    self.consume_token(TokenKind::Assign)?;

                    Ok(Statement::Assignment {
                        lhs: Assign::InstanceSet(
                            lhs,
                            calls
                                .into_iter()
                                .map(|s| AssignIndex::Identifier(s))
                                .collect(),
                        ),
                        expression: self.parse_expression()?,
                    }
                    .into())
                } else {
                    Err(ParsingError::ParseError(format!(
                        "Unexpected = after args in instance call - {lhs:?}.{} ({args:?})",
                        calls.join(".")
                    )))
                };
            }
            Ok(FunctionExpression::InstanceFunctionCall(Box::new(lhs), calls, args).into())
        } else {
            Ok(lhs.into())
        }
    }

    fn parse_value_expression(&mut self, value: TokenValue) -> Result<Expression, ParsingError> {
        self.parse_inline_expression(value)
    }

    fn parse_this_element(&mut self) -> Result<Element, ParsingError> {
        self.parse_inline_element(Expression::This)
    }

    fn parse_this_expression(&mut self) -> Result<Expression, ParsingError> {
        self.parse_inline_expression(Expression::This)
    }

    fn parse_this_expression_skip_inline(&mut self) -> Result<Expression, ParsingError> {
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

    fn parse_symbol_expression(&mut self, symbol: Symbol<'t>) -> Result<Expression, ParsingError> {
        self.parse_inline_expression(symbol)
    }

    fn parse_unary_expression(&mut self, op: UnaryOperation) -> Result<Expression, ParsingError> {
        let exp = self.parse_expression()?;
        Ok(Expression::unary(op, exp))
    }

    fn parse_args(&mut self) -> Result<(RigzArguments, bool), ParsingError> {
        let mut args = Vec::new();
        let mut needs_comma = false;
        let mut named = None;
        let mut assign = false;
        loop {
            match self.peek_token() {
                None => break,
                Some(t) if t.terminal() => break,
                Some(t) => match t.kind {
                    TokenKind::Rparen
                    | TokenKind::Rbracket
                    | TokenKind::Rcurly
                    | TokenKind::End
                    // binary operations are handled within parse_expression
                    // todo this causes lambdas to require parens or {}
                    | TokenKind::BinOp(_)
                    | TokenKind::Pipe
                    | TokenKind::And
                    | TokenKind::Minus => break,
                    TokenKind::Identifier(id) => {
                        self.consume_token(TokenKind::Identifier(id))?;
                        match self.peek_token() {
                            None if named.is_none() => {
                                args.push(self.parse_identifier_expression(id)?);
                                needs_comma = true
                            }
                            None => {
                                return Err(ParsingError::ParseError(format!("Expected : after {id} {t:?}")))
                            }
                            Some(s) => {
                                match s.kind {
                                    TokenKind::Colon  => {
                                        self.consume_token(TokenKind::Colon)?;
                                        match &mut named {
                                            None => {
                                                named = Some(vec![(id.to_string(), self.parse_expression()?)]);
                                            }
                                            Some(v) => {
                                                v.push((id.to_string(), self.parse_expression()?));
                                                needs_comma = true
                                            }
                                        }
                                    }
                                    TokenKind::Assign => {
                                        assign = true;
                                        break;
                                    }
                                    _ => {
                                        args.push(self.parse_identifier_expression(id)?);
                                        needs_comma = true
                                    }
                                }
                            }
                        };
                    }
                    TokenKind::Comma => {
                        self.consume_token(TokenKind::Comma)?;
                        needs_comma = false;
                        continue;
                    }
                    TokenKind::If | TokenKind::Unless if !needs_comma => {
                        // todo this needs to be way more efficient
                        let t = self.tokens.clone();
                        match self.parse_expression() {
                            Ok(e) => {
                                args.push(e);
                                needs_comma = true
                            }
                            Err(_) => {
                                self.tokens = t;
                                break
                            }
                        }
                    }
                    t if named.is_none() && !needs_comma => {
                        if t == TokenKind::Assign {
                            assign = true;
                            break
                        }
                        args.push(self.parse_expression()?);
                        needs_comma = true
                    }
                    _ if named.is_some() && !needs_comma => {
                        return Err(ParsingError::ParseError(format!("Positional args cannot be used after named args {t:?}")))
                    },
                    _ => break
                },
            }
        }

        let args = match named {
            None => {
                if args.len() == 1 {
                    let args = match args.remove(0) {
                        Expression::Tuple(a) => a.into(),
                        a => vec![a].into(),
                    };
                    args
                } else {
                    args.into()
                }
            }
            Some(n) if args.is_empty() => RigzArguments::Named(n),
            Some(n) => RigzArguments::Mixed(args, n),
        };
        Ok((args, assign))
    }

    fn parse_for_list(&mut self) -> Result<Expression, ParsingError> {
        let var = self.required_identifier()?;
        self.consume_token(TokenKind::In)?;
        let expression = self.parse_expression()?;
        self.consume_token_eat_newlines(TokenKind::Colon)?;
        let body = self.parse_expression()?;
        self.consume_token_eat_newlines(TokenKind::Rbracket)?;
        Ok(Expression::ForList {
            var,
            expression: Box::new(expression),
            body: Box::new(body),
        })
    }

    fn parse_list(&mut self) -> Result<Expression, ParsingError> {
        let next = self.peek_required_token_eat_newlines("parse_list")?;
        if next.kind == TokenKind::For {
            self.consume_token(TokenKind::For)?;
            return self.parse_for_list();
        }

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
        Ok(args.into())
    }

    fn required_identifier(&mut self) -> Result<String, ParsingError> {
        let t = self.next_required_token("required_identifier")?;
        match t.kind {
            TokenKind::Identifier(id) => Ok(id.to_string()),
            _ => Err(ParsingError::ParseError(format!(
                "Expected identifier got {t:?}"
            ))),
        }
    }

    fn parse_for_map(&mut self) -> Result<Expression, ParsingError> {
        let k_var = self.required_identifier()?;
        self.consume_token(TokenKind::Comma)?;
        let v_var = self.required_identifier()?;
        self.consume_token(TokenKind::In)?;
        let expression = self.parse_expression()?;
        self.consume_token(TokenKind::Colon)?;
        let key = self.parse_expression()?;
        let next = self.next_required_token("parse_for_map")?;
        let value = match next.kind {
            TokenKind::Comma => {
                let e = self.parse_expression()?;
                self.consume_token(TokenKind::Rcurly)?;
                Some(Box::new(e))
            }
            TokenKind::Rcurly => None,
            _ => {
                return Err(ParsingError::ParseError(format!(
                    "Expected , or }}, received {next:?}"
                )))
            }
        };

        Ok(Expression::ForMap {
            k_var,
            v_var,
            expression: Box::new(expression),
            key: Box::new(key),
            value,
        })
    }

    fn parse_map(&mut self) -> Result<Expression, ParsingError> {
        let next = self.peek_required_token("parse_map")?;
        match next.kind {
            TokenKind::For => {
                self.consume_token(TokenKind::For)?;
                return self.parse_for_map();
            }
            TokenKind::Pipe => {
                self.consume_token(next.kind)?;
                let lambda = self.parse_lambda(false)?;
                self.consume_token(TokenKind::Rcurly)?;
                return Ok(lambda);
            }
            TokenKind::BinOp(BinaryOperation::Or) => {
                self.consume_token(next.kind)?;
                let lambda = self.parse_lambda(true)?;
                self.consume_token(TokenKind::Rcurly)?;
                return Ok(lambda);
            }
            _ => {}
        }

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
                Some(_) => {
                    let key = self.parse_expression()?;
                    let t = self.next_required_token("parse_map: '=', ',', or '}' expected")?;
                    match t.kind {
                        TokenKind::Assign => {
                            let value = self.parse_expression()?;
                            args.push((key, value));
                        }
                        TokenKind::Comma => {
                            if let Expression::Identifier(id) = &key {
                                args.push((Expression::Value(id.as_str().into()), key));
                            } else {
                                args.push((key.clone(), key));
                            }
                        }
                        TokenKind::Rcurly => {
                            args.push((key.clone(), key));
                            break;
                        }
                        _ => {
                            return Err(ParsingError::ParseError(format!(
                                "Invalid Map Token {t:?}"
                            )))
                        }
                    }
                }
            }
        }
        Ok(Expression::Map(args))
    }

    fn parse_function_definition(
        &mut self,
        lifecycle: Option<Lifecycle>,
    ) -> Result<FunctionDefinition, ParsingError> {
        match self.parse_function_declaration()? {
            FunctionDeclaration::Declaration { name, .. } => Err(ParsingError::ParseError(
                format!("Missing body for function definition {name}"),
            )),
            FunctionDeclaration::Definition(mut f) => match lifecycle {
                None => Ok(f),
                Some(l) => {
                    if matches!(l, Lifecycle::On(_))
                        && f.type_definition.arg_type != ArgType::Positional
                    {
                        return Err(ParsingError::ParseError(format!(
                            "Positional arguments are required for @on lifecycle - {f:?}"
                        )));
                    }
                    f.lifecycle = Some(l);
                    Ok(f)
                }
            },
        }
    }

    fn parse_function_arguments(
        &mut self,
    ) -> Result<(Vec<FunctionArgument>, Option<usize>, ArgType), ParsingError> {
        let mut args = Vec::new();
        let next = self.peek_required_token_eat_newlines("parse_function_arguments")?;
        if !(next.kind == TokenKind::Lparen
            || next.kind == TokenKind::Lcurly
            || next.kind == TokenKind::Lbracket)
        {
            return Ok((args, None, ArgType::Positional));
        }

        let (terminal, arg_type) = match next.kind {
            TokenKind::Lparen => (TokenKind::Rparen, ArgType::Positional),
            TokenKind::Lcurly => (TokenKind::Rcurly, ArgType::Map),
            TokenKind::Lbracket => (TokenKind::Rbracket, ArgType::List),
            _ => unreachable!(),
        };
        self.consume_token(next.kind)?;

        let mut var_arg_start = None;
        self.parse_function_arguments_inner(&mut args, terminal, &mut var_arg_start)?;
        Ok((args, var_arg_start, arg_type))
    }

    fn parse_function_arguments_inner(
        &mut self,
        args: &mut Vec<FunctionArgument>,
        terminal: TokenKind<'t>,
        var_arg_start: &mut Option<usize>,
    ) -> Result<(), ParsingError> {
        loop {
            match self.peek_token() {
                None => break,
                Some(t) if t.kind == terminal => {
                    self.consume_token(terminal)?;
                    break;
                }
                Some(t) if t.kind == TokenKind::Comma => {
                    self.consume_token(TokenKind::Comma)?;
                    continue;
                }
                Some(_) => {
                    let arg = self.parse_function_argument(var_arg_start.is_some())?;
                    if arg.var_arg {
                        *var_arg_start = Some(args.len());
                    }
                    args.push(arg);
                }
            }
        }
        Ok(())
    }

    fn parse_lambda_arguments(
        &mut self,
    ) -> Result<(Vec<FunctionArgument>, Option<usize>), ParsingError> {
        let mut args = Vec::new();

        let mut var_arg_start = None;
        self.parse_function_arguments_inner(&mut args, TokenKind::Pipe, &mut var_arg_start)?;
        Ok((args, var_arg_start))
    }

    fn check_var_arg(&mut self, existing_var_arg: bool) -> Result<bool, ParsingError> {
        let next = self.peek_required_token("check_var_arg")?;
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

    fn parse_function_argument(
        &mut self,
        existing_var_arg: bool,
    ) -> Result<FunctionArgument, ParsingError> {
        // todo support mut, vm changes required
        let var_arg = self.check_var_arg(existing_var_arg)?;
        let next = self.next_required_token("parse_function_argument")?;
        match next.kind {
            TokenKind::Identifier(name) => self.parse_identifier_argument(var_arg, name, false),
            TokenKind::Type => self.parse_identifier_argument(var_arg, "rigz_type", false),
            TokenKind::Range => {
                let next = self.next_required_token("parse_function_argument - Range")?;
                if let TokenKind::Identifier(arg) = next.kind {
                    self.parse_identifier_argument(var_arg, arg, true)
                } else {
                    // todo should a named variable always be required?
                    Err(ParsingError::ParseError(format!(
                        "Invalid Function Argument after .. {:?}",
                        next
                    )))
                }
            }
            _ => Err(ParsingError::ParseError(format!(
                "Invalid Function Argument {:?}",
                next
            ))),
        }
    }

    fn parse_identifier_argument(
        &mut self,
        var_arg: bool,
        name: &'t str,
        rest: bool,
    ) -> Result<FunctionArgument, ParsingError> {
        let mut default_type = true;
        let next = self.peek_required_token("parse_identifier_argument")?;
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

        let default = match self
            .peek_required_token("parse_identifier_argument - default_value")?
            .kind
        {
            TokenKind::Assign => {
                self.consume_token(TokenKind::Assign)?;
                let v = self.parse_expression()?;
                if default_type {
                    if let Expression::Value(v) = &v {
                        rigz_type = v.rigz_type()
                    };
                }
                Some(v.into())
            }
            _ => None,
        };

        Ok(FunctionArgument {
            name: name.to_string(),
            default,
            function_type: rigz_type.into(),
            var_arg,
            rest,
        })
    }

    fn parse_return_type(&mut self, mut_self: bool) -> Result<FunctionType, ParsingError> {
        let mut rigz_type = if mut_self {
            RigzType::This
        } else {
            RigzType::default()
        };
        let mut mutable = mut_self;
        match self.peek_token() {
            None => return Err(Self::eoi_error_string("parse_return_type".to_string())),
            Some(t) => {
                if t.kind == TokenKind::Arrow {
                    self.consume_token(TokenKind::Arrow)?;
                    if self.peek_required_token("parse_return_type")?.kind == TokenKind::Mut {
                        self.consume_token(TokenKind::Mut)?;
                        mutable = true;
                    }
                    rigz_type = self.parse_rigz_type(None, false)?
                }
            }
        }
        Ok(FunctionType { rigz_type, mutable })
    }

    fn parse_rigz_type(
        &mut self,
        name: Option<&'t str>,
        paren: bool,
    ) -> Result<RigzType, ParsingError> {
        let next = self.next_required_token("parse_rigz_type")?;
        let rigz_type: RigzType = match next.kind {
            TokenKind::TypeValue(id) => match id.parse::<RigzType>() {
                Ok(t) => t,
                Err(e) => {
                    return Err(ParsingError::ParseError(format!(
                        "Invalid type value {:?}",
                        e
                    )))
                }
            },
            TokenKind::Lbracket => {
                let t = self.peek_required_token("parse_rigz_type - [")?;
                match t.kind {
                    TokenKind::Rbracket => {
                        self.consume_token(TokenKind::Rbracket)?;
                        RigzType::List(Box::default())
                    }
                    TokenKind::TypeValue(_) => {
                        let l = RigzType::List(Box::new(self.parse_rigz_type(None, paren)?));
                        self.consume_token(TokenKind::Rbracket)?;
                        l
                    }
                    _ => {
                        return Err(ParsingError::ParseError(format!(
                            "Invalid list type {:?}",
                            t
                        )))
                    }
                }
            }
            TokenKind::Lcurly => {
                let mut key_type = None;
                let mut value_type = None;
                let mut custom_type = None;
                loop {
                    let t = self.peek_required_token("parse_rigz_type - {")?;
                    if t.terminal() {
                        self.consume_token(t.kind)?;
                        continue;
                    }
                    match t.kind {
                        TokenKind::Identifier(_) if name.is_some() => {
                            custom_type = Some(self.parse_custom_type(name.unwrap())?);
                            break;
                        }
                        TokenKind::Rcurly => {
                            self.consume_token(TokenKind::Rcurly)?;
                            break;
                        }
                        TokenKind::TypeValue(_) if key_type.is_none() => {
                            key_type = Some(self.parse_rigz_type(None, paren)?);
                        }
                        TokenKind::Comma if key_type.is_some() => {
                            self.consume_token(TokenKind::Comma)?;
                            value_type = Some(self.parse_rigz_type(None, paren)?);
                        }
                        _ => {
                            return Err(ParsingError::ParseError(format!(
                                "Invalid map type {:?}",
                                t
                            )))
                        }
                    }
                }

                match custom_type {
                    None => match (key_type, value_type) {
                        (None, None) => RigzType::Map(Box::default(), Box::default()),
                        (Some(t), None) => RigzType::Map(Box::new(t.clone()), Box::new(t)),
                        (Some(k), Some(v)) => RigzType::Map(Box::new(k), Box::new(v)),
                        _ => unreachable!(),
                    },
                    Some(t) => RigzType::Custom(t),
                }
            }
            TokenKind::Lparen => {
                let mut t = self.parse_rigz_type(None, true)?;
                loop {
                    let next = self.peek_required_token("parse_rigz_type - (")?;
                    match next.kind {
                        TokenKind::Comma => {
                            self.consume_token(TokenKind::Comma)?;
                        }
                        TokenKind::Rparen => {
                            self.consume_token(TokenKind::Rparen)?;
                            break;
                        }
                        _ => match &mut t {
                            RigzType::Tuple(v) => v.push(self.parse_rigz_type(None, true)?),
                            next => t = RigzType::Tuple(vec![next.clone()]),
                        },
                    }
                }
                t
            }
            TokenKind::Pipe => {
                let mut args = vec![];
                loop {
                    let next = self.peek_required_token("parse_rigz_type - |")?;
                    match next.kind {
                        TokenKind::Pipe => {
                            self.consume_token(TokenKind::Pipe)?;
                            break;
                        }
                        TokenKind::Comma => {
                            self.consume_token(TokenKind::Comma)?;
                        }
                        _ => args.push(self.parse_rigz_type(None, false)?),
                    }
                }
                let FunctionType { rigz_type, .. } = self.parse_return_type(false)?;
                RigzType::Function(args, Box::new(rigz_type))
            }
            TokenKind::BinOp(BinaryOperation::Or) => {
                let FunctionType { rigz_type, .. } = self.parse_return_type(false)?;
                RigzType::Function(vec![], Box::new(rigz_type))
            }
            _ => return Err(ParsingError::ParseError(format!("Invalid type {:?}", next))),
        };

        self.parse_type_suffix(rigz_type, paren)
    }

    fn parse_type_suffix(
        &mut self,
        rigz_type: RigzType,
        paren: bool,
    ) -> Result<RigzType, ParsingError> {
        let rt = match self.peek_token() {
            None => rigz_type,
            Some(t) => match t.kind {
                TokenKind::BinOp(BinaryOperation::Or) => {
                    RigzType::Union(self.parse_complex_type(rigz_type, true, paren)?)
                }
                TokenKind::And => {
                    RigzType::Composite(self.parse_complex_type(rigz_type, false, paren)?)
                }
                TokenKind::Optional => {
                    self.consume_token(TokenKind::Optional)?;
                    let can_return_error = match self.peek_token() {
                        None => false,
                        Some(t) if t.kind == TokenKind::Not => {
                            self.consume_token(TokenKind::Not)?;
                            true
                        }
                        Some(_) => false,
                    };
                    RigzType::Wrapper {
                        base_type: Box::new(rigz_type),
                        optional: true,
                        can_return_error,
                    }
                }
                TokenKind::Not => {
                    self.consume_token(TokenKind::Not)?;
                    RigzType::Wrapper {
                        base_type: Box::new(rigz_type),
                        optional: false,
                        can_return_error: true,
                    }
                }
                _ => rigz_type,
            },
        };
        Ok(rt)
    }

    fn parse_custom_type(&mut self, name: &'t str) -> Result<CustomType, ParsingError> {
        let mut fields = vec![];
        let mut needs_separator = false;
        loop {
            let t = self.peek_required_token("parse_custom_type")?;
            match t.kind {
                TokenKind::Identifier(id) => {
                    self.consume_token(TokenKind::Identifier(id))?;
                    self.consume_token(TokenKind::Colon)?;
                    fields.push((id.to_string(), self.parse_rigz_type(None, false)?));
                    needs_separator = true;
                }
                TokenKind::Rcurly => {
                    self.consume_token(TokenKind::Rcurly)?;
                    break;
                }
                TokenKind::Comma if needs_separator => {
                    self.consume_token(TokenKind::Comma)?;
                    needs_separator = false;
                }
                _ if t.terminal() => self.consume_token(t.kind)?,
                _ => {
                    return Err(ParsingError::ParseError(format!(
                        "Invalid token for custom type {t:?}"
                    )));
                }
            }
        }
        Ok(CustomType {
            name: name.to_string(),
            fields,
        })
    }

    fn parse_complex_type(
        &mut self,
        rigz_type: RigzType,
        union: bool,
        paren: bool,
    ) -> Result<Vec<RigzType>, ParsingError> {
        if union {
            self.consume_token(TokenKind::BinOp(BinaryOperation::Or))?;
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
                    break;
                }
                Some(t) if needs_sep => {
                    if t.kind == TokenKind::Rparen && paren {
                        break;
                    }
                    match t.kind {
                        TokenKind::Assign => break,
                        _ => {
                            let separator = if union {
                                TokenKind::BinOp(BinaryOperation::Or)
                            } else {
                                TokenKind::And
                            };
                            if t.kind == separator {
                                self.consume_token(separator)?;
                                needs_sep = false;
                            } else {
                                return Err(ParsingError::ParseError(format!(
                                    "Unexpected token in {} type - {t:?}",
                                    if union { "union" } else { "composite" }
                                )));
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
                        t => complex.push(t),
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
    ) -> Result<FunctionSignature, ParsingError> {
        let (arguments, var_args_start, arg_type) = self.parse_function_arguments()?;
        Ok(FunctionSignature {
            arguments,
            var_args_start,
            return_type: self.parse_return_type(mut_self)?,
            arg_type,
            self_type: None,
        })
    }

    fn parse_scope(&mut self) -> Result<Scope, ParsingError> {
        let mut elements = vec![];
        loop {
            let next = self.peek_required_token_eat_newlines("parse_scope")?;
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

    fn parse_if_scope(&mut self) -> Result<(Scope, Option<Scope>), ParsingError> {
        let mut elements = vec![];
        let mut else_encountered = false;
        loop {
            let next = self.peek_required_token("parse_if_scope")?;
            match next.kind {
                TokenKind::End => {
                    if elements.is_empty() {
                        return Err(ParsingError::ParseError(format!(
                            "Missing end for if scope: {next:?}"
                        )));
                    }
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

    fn parse_trait_definition(&mut self) -> Result<TraitDefinition, ParsingError> {
        let next = self.next_required_token("parse_trait_definition")?;
        let name = if let TokenKind::TypeValue(name) = next.kind {
            name.to_string()
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

    fn parse_trait_declarations(&mut self) -> Result<Vec<FunctionDeclaration>, ParsingError> {
        let mut all = Vec::new();
        loop {
            let next = self.peek_required_token_eat_newlines("parse_trait_declarations")?;
            match next.kind {
                TokenKind::End => break,
                TokenKind::FunctionDef => {
                    self.consume_token(TokenKind::FunctionDef)?;
                    let def = self.peek_required_token("parse_trait_declarations - fn")?;
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

    fn parse_function_declaration(&mut self) -> Result<FunctionDeclaration, ParsingError> {
        let next = self.peek_required_token("parse_function_declaration")?;
        match next.kind {
            TokenKind::Mut => {
                self.consume_token(TokenKind::Mut)?;
                let next = self.next_required_token("parse_function_declaration - mut")?;

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
        rigz_type: Option<&'t str>,
        mutable: bool,
    ) -> Result<FunctionDeclaration, ParsingError> {
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
                        "Invalid fn type: {} {:?}",
                        rt, e
                    )))
                }
            },
            None => None,
        };
        let next = self.next_required_token("parse_typed_function_declaration")?;

        let name = match next.kind {
            TokenKind::Type => {
                // hack to support type as function name
                "type"
            }
            TokenKind::Identifier(name)
                if matches!(
                    name,
                    "send" | "receive" | "log" | "puts" | "spawn" | "broadcast" | "sleep"
                ) =>
            {
                return Err(ParsingError::ParseError(format!(
                    "{name} is a reserved function name and cannot be overwritten"
                )))
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
        let next = self.peek_required_token_eat_newlines("parse_typed_function_declaration")?;
        let dec = match next.kind {
            TokenKind::FunctionDef | TokenKind::End => FunctionDeclaration::Declaration {
                name: name.to_string(),
                type_definition,
            },
            _ => FunctionDeclaration::Definition(FunctionDefinition {
                name: name.to_string(),
                type_definition,
                body: self.parse_scope()?,
                lifecycle: None,
            }),
        };
        Ok(dec)
    }

    pub fn parse_object_definition(&mut self) -> Result<ObjectDefinition, ParsingError> {
        self.consume_token(TokenKind::Object)?;
        let n = self.next_required_token("parse_object_definition")?;
        let name = if let TokenKind::TypeValue(ty) = n.kind {
            ty.to_string()
        } else {
            return Err(ParsingError::ParseError(format!(
                "Missing Type value for Object {n:?}"
            )));
        };
        let fields = self.parse_attrs()?;
        let rigz_type = RigzType::Custom(CustomType {
            name,
            fields: fields
                .iter()
                .map(|f| (f.name.clone(), f.attr_type.rigz_type.clone()))
                .collect(),
        });

        let constructor = self.parse_constructor()?;
        let functions = self.parse_trait_declarations()?;
        self.consume_token_eat_newlines(TokenKind::End)?;
        Ok(ObjectDefinition {
            rigz_type,
            fields,
            constructor,
            functions,
        })
    }

    pub fn parse_constructor(&mut self) -> Result<Constructor, ParsingError> {
        let t = self.peek_required_token("parse_constructor - end required")?;
        if let TokenKind::TypeValue(tv) = t.kind {
            if tv != "Self" {
                return Err(ParsingError::ParseError(format!("Received non-self type for constructor, {tv}, use Self() or rely on default constructor")));
            }
            self.consume_token(t.kind)?;
            let (args, var, ty) = self.parse_function_arguments()?;
            // todo support all types for ty
            let next = self.peek_required_token_eat_newlines("parse_constructor - fn or end")?;
            return if let TokenKind::FunctionDef = next.kind {
                Ok(Constructor::Declaration(args, var))
            } else {
                Ok(Constructor::Definition(args, var, self.parse_scope()?))
            };
        }
        Ok(Constructor::Default)
    }

    pub fn parse_attrs(&mut self) -> Result<Vec<ObjectAttr>, ParsingError> {
        let mut attrs = Vec::new();
        loop {
            let next = self.peek_required_token_eat_newlines("parse_attrs - attr")?;
            // todo support mut attr
            if next.kind == TokenKind::Attr {
                self.consume_token(next.kind)?;
            } else {
                break;
            }

            let next = self.next_required_token("parse_attrs - id")?;
            let id = if let TokenKind::Identifier(id) = next.kind {
                id
            } else {
                return Err(ParsingError::ParseError(format!(
                    "Expected identifier after `attr`, received {next:?}"
                )));
            };

            let comma = self.peek_required_token("parse_attrs - end required")?;
            let rt = if comma.kind == TokenKind::Comma {
                self.consume_token(comma.kind)?;
                self.parse_rigz_type(None, false)?
            } else {
                RigzType::default()
            };

            let def = self.peek_required_token("parse_attrs - end required")?;
            let default = if def.kind == TokenKind::Assign {
                self.consume_token(def.kind)?;
                Some(self.parse_expression()?)
            } else {
                None
            };

            attrs.push(ObjectAttr {
                name: id.to_string(),
                attr_type: FunctionType {
                    rigz_type: rt,
                    mutable: false,
                },
                default,
            });
        }
        Ok(attrs)
    }
}

fn convert_to_assign(tuple: &mut Vec<Expression>) -> Result<Vec<(String, bool)>, ParsingError> {
    let mut results = Vec::with_capacity(tuple.len());
    for e in tuple.iter() {
        match e {
            Expression::Identifier(id) => {
                results.push((id.to_string(), false));
            }
            Expression::Tuple(t) => {
                return Err(ParsingError::ParseError(format!(
                    "nested tuples not supported yet - {t:?}"
                )))
            }
            _ => {
                return Err(ParsingError::ParseError(format!(
                    "Expression found in tuple assign {e:?}"
                )))
            }
        }
    }
    tuple.clear();
    Ok(results)
}
