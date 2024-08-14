use std::collections::{HashMap, HashSet};
use indexmap::IndexMap;
use logos::{Lexer, Logos};
use rigz_vm::{Binary, BinaryOperation, Instruction, Register, RigzType, Unary, UnaryOperation, VMBuilder, Value, VM};
use crate::FunctionDefinition;
use crate::token::{LexingError, Token, TokenKind};
use crate::token::TokenKind::BinOp;

pub struct Parser<'lex> {
    lexer: Lexer<'lex, TokenKind<'lex>>,
    builder: VMBuilder<'lex>,
    function_declarations: HashMap<String, FunctionDefinition>,
    next: Register,
    last: Register,
    current_token: Option<Token<'lex>>,
}

impl <'lex> Parser<'lex> {
    fn build(&mut self) -> VM<'lex> {
        self.builder.build()
    }

    fn build_multiple(&mut self) -> VM<'lex> {
        let (vm, _) = self.builder.build_multiple();
        vm
    }
}

pub struct VMParser<'lex> {
    lexer: Lexer<'lex, TokenKind<'lex>>,
    builder: VM<'lex>,
    function_declarations: HashMap<String, FunctionDefinition>,
    next: Register,
    last: Register,
    current_token: Option<Token<'lex>>,
}

impl <'lex> VMParser<'lex> {
    fn build(&mut self) -> VM<'lex> {
        std::mem::take(&mut self.builder)
    }

    fn build_multiple(&mut self) -> VM<'lex> {
        self.builder.clone()
    }
}

#[derive(Clone, Debug)]
pub enum Statement<'lex> {
    Assignment {
        name: &'lex str,
        mutable: bool,
        expression: Expression<'lex>
    },
    FunctionDefinition {
        name: &'lex str,
        type_definition: FunctionDefinition,
        elements: Vec<Element<'lex>>
    },
    Expression(Expression<'lex>)
    // import, exports
}

#[derive(Clone, Debug)]
pub enum Expression<'lex> {
    Value(Value<'lex>),
    List(Vec<Expression<'lex>>),
    Map(IndexMap<Expression<'lex>, Expression<'lex>>),
    Identifier(&'lex str),
    BinExp(Box<Expression<'lex>>, BinaryOperation, Box<Expression<'lex>>),
    UnaryExp(UnaryOperation, Box<Expression<'lex>>),
    FunctionCall(&'lex str, Vec<Expression<'lex>>),
    Scope(Vec<Expression<'lex>>)
}

#[derive(Clone, Debug)]
pub enum Element<'lex> {
    Statement(Statement<'lex>),
    Expression(Expression<'lex>)
}

macro_rules! gen_parser {
    ($type:ident, $builder:ident, $init:expr) => {
impl <'lex> $type<'lex> {
    pub fn parse_with_builder(input: &'lex str, builder: $builder<'lex>) -> Result<VM<'lex>, LexingError> {
    //pub fn parse_with_builder(input: &'lex str, builder: VMBuilder<'lex>) -> Result<VM<'lex>, LexingError> {
        let lexer = TokenKind::lexer(input);
        let mut parser = Self {
            lexer,
            builder,
            function_declarations: HashMap::from([
                ("puts".to_string(), FunctionDefinition {
                    arguments: vec![RigzType::Any],
                    return_type: RigzType::None,
                }),
                ("eprint".to_string(), FunctionDefinition {
                    arguments: vec![RigzType::Any],
                    return_type: RigzType::None,
                }),
            ]),
            next: 2,
            last: 0,
            current_token: None,
        };

        loop {
            let next = parser.next_token()?;

            let element = match next {
                None => break,
                Some(t) => {
                    println!("token: {:?}", t);
                    parser.next_element(t)?
                }
            };

            match element {
                None => {}
                Some(e) => {
                    parser.build_element(e);
                }
            }
        }

        if parser.builder.sp == 0 {
            parser.builder.add_halt_instruction(parser.last);
        }

        Ok(parser.build())
    }

    pub fn parse(input: &'lex str) -> Result<VM<'lex>, LexingError> {
        Self::parse_with_builder(input, $init())
    }

    pub fn next_expression(&mut self, token: Option<Token<'lex>>) -> Result<Expression<'lex>, LexingError> {
        match token {
            None => Err(LexingError::ParseError("Missing expression".to_string())),
            Some(t) => {
                let kind = t.kind.clone();
                let element = self.next_element(t)?;
                match element {
                    None => {
                        Err(LexingError::ParseError(format!("No element after {:?}", kind)))
                    },
                    Some(e) => match e {
                        Element::Statement(s) => Err(LexingError::ParseError(format!("Unexpected statement {:?}", s))),
                        Element::Expression(e) => Ok(e),
                    },
                }
            }
        }
    }

    pub fn handle_identifier(&mut self, id: &'lex str) -> Result<Option<Element<'lex>>, LexingError> {
        match self.next_token()? {
            None => Ok(Some(Element::Expression(Expression::Identifier(id)))),
            Some(t) => {
                match t.kind {
                    TokenKind::Newline => Ok(Some(Element::Expression(Expression::Identifier(id)))),
                    TokenKind::Value(v) => Err(LexingError::ParseError(format!("Unexpected Value {}", v))),
                    TokenKind::Assign => {
                        let token = self.next_token()?;
                        let expr = self.next_expression(token)?;
                        Ok(Some(Element::Statement(Statement::Assignment {
                            name: id,
                            mutable: false,
                            expression: expr
                        })))
                    }
                    TokenKind::BinOp(op) => {
                        let token = self.next_token()?;
                        let expr = self.next_expression(token)?;
                        Ok(Some(Element::Expression(Expression::BinExp(Box::new(Expression::Identifier(id)), op, Box::new(expr)))))
                    }
                    TokenKind::Minus => {
                        let token = self.next_token()?;
                        let expr = self.next_expression(token)?;
                        Ok(Some(Element::Expression(Expression::BinExp(Box::new(Expression::Identifier(id)), BinaryOperation::Sub, Box::new(expr)))))
                    }
                    TokenKind::Period => {
                        todo!() // maybe valid for extension function?
                    }
                    k => Err(LexingError::ParseError(format!("Unexpected {:?} after identifier", k))),
                }
            }
        }
    }

    pub fn next_element(&mut self, token: Token<'lex>) -> Result<Option<Element<'lex>>, LexingError> {
        match token.kind {
            TokenKind::Newline => Ok(None),
            TokenKind::Value(v) => {
                match self.next_token()? {
                    None => {
                        Ok(Some(Element::Expression(Expression::Value(v))))
                    }
                    Some(s) => {
                        match s.kind {
                            TokenKind::Newline => Ok(Some(Element::Expression(Expression::Value(v)))),
                            TokenKind::Semi => Ok(Some(Element::Expression(Expression::Value(v)))),
                            TokenKind::Value(v) => Err(LexingError::ParseError(format!("Unexpected value {}", v))),
                            TokenKind::BinOp(o) => {
                                let next = self.next_token()?;
                                let expr = self.next_expression(next)?;
                                Ok(Some(Element::Expression(Expression::BinExp(Box::new(Expression::Value(v)), o, Box::new(expr)))))
                            }
                            TokenKind::Minus => {
                                let next = self.next_token()?;
                                let expr = self.next_expression(next)?;
                                Ok(Some(Element::Expression(Expression::BinExp(Box::new(Expression::Value(v)), BinaryOperation::Sub, Box::new(expr)))))
                            }
                            TokenKind::Period => {
                                todo!() // value extension functions
                            }
                            k => Err(LexingError::ParseError(format!("Unexpected value {:?}", k))),
                        }
                    }
                }
            },
            TokenKind::Assign => Err(LexingError::ParseError("Unexpected =".to_string())),
            TokenKind::BinOp(b) => Err(LexingError::ParseError(format!("Unexpected {:?}", b))),
            TokenKind::Not => {
                let token = self.next_token()?;
                let next = self.next_expression(token)?;
                Ok(Some(Element::Expression(Expression::UnaryExp(UnaryOperation::Not, Box::new(next)))))
            }
            TokenKind::Minus => {
                let token = self.next_token()?;
                let next = self.next_expression(token)?;
                Ok(Some(Element::Expression(Expression::UnaryExp(UnaryOperation::Neg, Box::new(next)))))
            }
            TokenKind::Period => Err(LexingError::ParseError("Unexpected .".to_string())),
            TokenKind::Comma => Err(LexingError::ParseError("Unexpected ,".to_string())),
            TokenKind::FunctionDef => {
                // fn <FunctionIdentifier> ( arg (:type)? ) (type)?
                //  statements*
                //  expression
                // end
                todo!()
            }
            TokenKind::Identifier(id) => {
                self.handle_identifier(id)
            }
            TokenKind::FunctionIdentifier(id) => {
                if self.function_declarations.contains_key(id) {
                    let mut args = Vec::new();
                    let mut last = None;
                    loop {
                        let next = self.next_token()?;
                        match next {
                            None => break,
                            Some(t) if t.kind == TokenKind::Comma && last != Some(TokenKind::Comma) => {
                                last = Some(t.kind);
                            }
                            Some(t) if t.kind == TokenKind::Comma && last == Some(TokenKind::Comma) => {
                                return Err(LexingError::ParseError("Unexpected ,".to_string()))
                            }
                            Some(t) => {
                                let expr = self.next_expression(Some(t))?;
                                args.push(expr);
                            }
                        }
                    }
                    Ok(Some(Element::Expression(Expression::FunctionCall(id, args))))
                } else {
                    self.handle_identifier(id)
                }
            }
            TokenKind::Lparen => {
                todo!() // create scope
            }
            TokenKind::Rparen => Err(LexingError::ParseError("Unexpected )".to_string())),
            TokenKind::Lcurly => {
                let mut map = IndexMap::new();
                Ok(Some(Element::Expression(Expression::Map(map))))
            }
            TokenKind::Rcurly => Err(LexingError::ParseError("Unexpected }".to_string())),
            TokenKind::Lbracket => {
                let mut list = Vec::new();
                loop {
                    let next = self.next_token()?;
                    match next {
                        None => break,
                        Some(t) => {
                            if TokenKind::Comma == t.kind {
                                let next = self.next_token()?;
                                match next {
                                    None => continue,
                                    Some(t) => {
                                        if TokenKind::Comma == t.kind {
                                            return Err(LexingError::ParseError("Unexpected ,".to_string()))
                                        }
                                        self.current_token = Some(t)
                                    }
                                }
                                continue
                            } else {
                                list.push(self.next_expression(Some(t))?);
                            }
                        }
                    }
                }
                Ok(Some(Element::Expression(Expression::List(list))))
            }
            TokenKind::Rbracket => Err(LexingError::ParseError("Unexpected ]".to_string())),
            TokenKind::Do => {
                // consume next element until End, special case for fn definition and inner scopes
                todo!()
            }
            TokenKind::End => Err(LexingError::ParseError("Unexpected end".to_string())),
            TokenKind::Let => todo!(),
            TokenKind::Mut => todo!(),
            TokenKind::As => Err(LexingError::ParseError("Unexpected as".to_string())),
            TokenKind::Semi => Ok(None),
        }
    }

    pub fn set_last(&mut self, last: Register) {
        self.last = last;
    }

    pub fn next_register(&mut self) -> Register {
        let next = self.next;
        self.set_last(next);
        match self.next.checked_add(1) {
            None => panic!("Registers have exceeded u64::MAX"),
            Some(r) => {
                self.next = r;
            }
        };
        next
    }

    /// must call self.current_token = before next call
    pub fn next_token(&mut self) -> Result<Option<Token<'lex>>, LexingError> {
        let current = std::mem::take(&mut self.current_token);
        match current {
            None => {
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
            Some(t) => Ok(Some(t))
        }
    }

    fn load_value(&mut self, value: Value<'lex>) {
        let next = self.next_register();
        self.builder.add_load_instruction(next, value);
    }

    fn to_value(&mut self, expressions: Vec<Expression<'lex>>) -> Value<'lex> {
        match expressions.len() {
            0 => Value::None,
            1 => {
                match &expressions[0] {
                    Expression::Value(v) => v.clone(),
                    _ => todo!()
                }
            }
            _ => todo!()
        }
    }

    fn build_expression(&mut self, expression: Expression<'lex>) {
        match expression {
            Expression::Value(v) => self.load_value(v),
            Expression::List(_) => {}
            Expression::Map(_) => {}
            Expression::Identifier(i) => {
                let next = self.next_register();
                self.builder.add_get_variable_instruction(i.to_string(), next);
            }
            Expression::BinExp(lhs, op, rhs) => {
                self.build_expression(*lhs);
                let lhs = self.last;
                self.build_expression(*rhs);
                let rhs = self.last;
                let next = self.next_register();
                self.builder.add_instruction(Instruction::Binary(Binary{
                    op,
                    lhs,
                    rhs,
                    output: next
                }));
            }
            Expression::UnaryExp(op, expression) => {
                self.build_expression(*expression);
                let from = self.last;
                let output = self.next_register();
                self.builder.add_instruction(Instruction::Unary(Unary {
                    op,
                    from,
                    output,
                }));
            }
            Expression::FunctionCall(name, def) => {
                match name {
                    "puts" => {
                        let list = self.to_value(def);
                        self.load_value(list);
                        self.builder.add_print_instruction(self.last, 0);
                        self.set_last(0);
                    }
                    "eprintln" => {
                        let list = self.to_value(def);
                        self.load_value(list);
                        self.builder.add_eprint_instruction(self.last, 0);
                        self.set_last(0);
                    }
                    _ => {
                        todo!()
                    }
                }
            }
            Expression::Scope(s) => {
                self.builder.enter_scope();
                for expr in s {
                    self.build_expression(expr);
                }
                self.builder.exit_scope(self.last);
            }
        }
    }

    fn build_assignment_value(&mut self, name: &'lex str, mutable: bool, value: Value<'lex>) {
        self.load_value(value);
        if mutable {
            self.builder.add_load_mut_instruction(name.to_string(), self.last);
        } else {
            self.builder.add_load_let_instruction(name.to_string(), self.last);
        }
    }

    fn build_assignment_identifier(&mut self, name: &'lex str, mutable: bool, id: &'lex str) {
        let next = self.next_register();
        self.builder.add_get_variable_instruction(id.to_string(), next);
        if mutable {
            self.builder.add_load_mut_instruction(name.to_string(), self.last);
        } else {
            self.builder.add_load_let_instruction(name.to_string(), self.last);
        }
    }

    fn build_statement(&mut self, statement: Statement<'lex>) {
        match statement {
            Statement::Assignment { name, mutable, expression } => {
                match expression {
                    Expression::Value(v) => self.build_assignment_value(name, mutable, v),
                    Expression::Identifier(id) => self.build_assignment_identifier(name, mutable, id),
                    Expression::List(_) => {
                        // if list<value> store as value, otherwise create scope
                    }
                    Expression::Map(_) => {
                        // if list<value> store as value, otherwise create scope
                    }
                    Expression::BinExp(lhs, op, rhs) => {
                        self.builder.enter_scope();
                        let scope = self.builder.sp;
                        self.build_expression(*lhs);
                        let lhs = self.last;
                        self.build_expression(*rhs);
                        let rhs = self.last;
                        let output = self.next_register();
                        self.builder.add_instruction(Instruction::Binary(Binary{
                            op,
                            lhs,
                            rhs,
                            output
                        }));
                        self.builder.exit_scope(output);
                        self.load_value(Value::ScopeId(scope, output));
                        self.builder.add_load_let_instruction(name.to_string(), self.last);
                    }
                    Expression::UnaryExp(op, expr) => {
                        self.builder.enter_scope();
                        self.build_expression(*expr);
                        self.builder.exit_scope(self.last);
                        let from = self.last;
                        let output = self.next_register();
                        self.builder.add_instruction(Instruction::Unary(Unary {
                            op,
                            from,
                            output,
                        }));
                    }
                    Expression::FunctionCall(_, _) => {
                        // create scope
                    }
                    Expression::Scope(_) => {}
                }
            }
            Statement::FunctionDefinition { .. } => {}
            Statement::Expression(e) => self.build_expression(e),
        }
    }

    fn build_element(&mut self, element: Element<'lex>) {
        match element {
            Element::Statement(s) => self.build_statement(s),
            Element::Expression(e) => self.build_expression(e),
        }
    }
}
    };
}

gen_parser!(Parser, VMBuilder, VMBuilder::new);
gen_parser!(VMParser, VM, VM::new);