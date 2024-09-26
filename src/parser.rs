use crate::token::{LexingError, Token, TokenKind};
use crate::{FunctionArgument, FunctionDefinition};
use indexmap::IndexMap;
use logos::{Lexer, Logos};
use rigz_vm::{
    Binary, BinaryOperation, Instruction, Register, RigzType, Unary, UnaryOperation, VMBuilder,
    Value, VM,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct FunctionCallDefinition {
    scope: usize, // non zero
    args: Vec<usize>,
    output: usize // 0 if output is None
}

pub struct Parser<'lex> {
    lexer: Lexer<'lex, TokenKind<'lex>>,
    builder: VMBuilder<'lex>,
    function_declarations: HashMap<&'lex str, FunctionDefinition<'lex>>,
    function_scopes: HashMap<&'lex str, FunctionCallDefinition>,
    next: Register,
    last: Register,
    current_token: Option<Token<'lex>>,
}

impl<'lex> Parser<'lex> {
    fn build(&mut self) -> VM<'lex> {
        self.builder.build()
    }
}

impl <'lex> From<&FunctionArgument<'lex>> for RigzType {
    fn from(value: &FunctionArgument<'lex>) -> Self {
        value.rigz_type.clone()
    }
}

pub struct VMParser<'lex> {
    lexer: Lexer<'lex, TokenKind<'lex>>,
    builder: VM<'lex>,
    function_declarations: HashMap<String, FunctionDefinition<'lex>>,
    next: Register,
    last: Register,
    current_token: Option<Token<'lex>>,
}

impl<'lex> VMParser<'lex> {
    fn build(&mut self) -> VM<'lex> {
        std::mem::take(&mut self.builder)
    }
}

impl<'lex> Parser<'lex> {
// macro_rules! gen_parser {
//     ($type:ident, $builder:ident, $init:expr) => {
//         impl<'lex> $type<'lex> {
//             pub fn parse_with_builder(
//                 input: &'lex str,
//                 builder: $builder<'lex>,
//             ) -> Result<VM<'lex>, LexingError> {
    pub fn parse_with_builder(input: &'lex str, builder: VMBuilder<'lex>) -> Result<VM<'lex>, LexingError> {
                let lexer = TokenKind::lexer(input);
                let mut parser = Self {
                    lexer,
                    builder,
                    function_scopes: Default::default(),
                    function_declarations: HashMap::from([
                        (
                            "puts",
                            FunctionDefinition {
                                arguments: vec![FunctionArgument { name: None, rigz_type: RigzType::Any, default: None }] ,
                                return_type: RigzType::None,
                            },
                        ),
                        (
                            "eprint",
                            FunctionDefinition {
                                arguments: vec![FunctionArgument { name: None, rigz_type: RigzType::Any, default: None }],
                                return_type: RigzType::None,
                            },
                        ),
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
                        None => {} // why dont i break here?
                        Some(e) => {
                            parser.build_element(e)?;
                        }
                    }
                }

                if parser.builder.sp == 0 {
                    parser.builder.add_halt_instruction(parser.last);
                }

                Ok(parser.build())
            }

            pub fn parse(input: &'lex str) -> Result<VM<'lex>, LexingError> {
                //Self::parse_with_builder(input, $init())
                Self::parse_with_builder(input, VMBuilder::new())
            }

            pub fn next_expression(
                &mut self,
                token: Option<Token<'lex>>,
            ) -> Result<Expression<'lex>, LexingError> {
                match token {
                    None => Err(LexingError::ParseError("Missing expression".to_string())),
                    Some(t) => {
                        let kind = t.kind.clone();
                        let element = self.next_element(t)?;
                        match element {
                            None => Err(LexingError::ParseError(format!(
                                "No element after {:?}",
                                kind
                            ))),
                            Some(e) => match e {
                                Element::Statement(s) => Err(LexingError::ParseError(format!(
                                    "Unexpected statement {:?}",
                                    s
                                ))),
                                Element::Expression(e) => Ok(e),
                            },
                        }
                    }
                }
            }

            pub fn handle_identifier(
                &mut self,
                id: &'lex str,
            ) -> Result<Option<Element<'lex>>, LexingError> {
                match self.next_token()? {
                    None => Ok(Some(Element::Expression(Expression::Identifier(id)))),
                    Some(t) => match t.kind {
                        TokenKind::Newline => {
                            Ok(Some(Element::Expression(Expression::Identifier(id))))
                        }
                        TokenKind::Value(v) => {
                            Err(LexingError::ParseError(format!("Unexpected Value {}", v)))
                        }
                        TokenKind::Assign => {
                            let token = self.next_token()?;
                            let expr = self.next_expression(token)?;
                            Ok(Some(Element::Statement(Statement::Assignment {
                                name: id,
                                mutable: false,
                                expression: expr,
                            })))
                        }
                        TokenKind::BinOp(op) => {
                            let token = self.next_token()?;
                            let expr = self.next_expression(token)?;
                            Ok(Some(Element::Expression(Expression::BinExp(
                                Box::new(Expression::Identifier(id)),
                                op,
                                Box::new(expr),
                            ))))
                        }
                        TokenKind::Minus => {
                            let token = self.next_token()?;
                            let expr = self.next_expression(token)?;
                            Ok(Some(Element::Expression(Expression::BinExp(
                                Box::new(Expression::Identifier(id)),
                                BinaryOperation::Sub,
                                Box::new(expr),
                            ))))
                        }
                        TokenKind::Period => {
                            if self.builder.module_exists(id) {
                                let (func, args) = self.next_function_call()?;
                                Ok(Some(Element::Expression(Expression::ModuleCall(
                                    id, func, args,
                                ))))
                            } else {
                                todo!()
                            }
                        }
                        k => Err(LexingError::ParseError(format!(
                            "Unexpected {:?} after identifier",
                            k
                        ))),
                    },
                }
            }

            pub fn next_function_call(
                &mut self,
            ) -> Result<(&'lex str, Vec<Expression<'lex>>), LexingError> {
                let t = match self.next_token()? {
                    None => return Err(LexingError::ParseError("Expected function call".into())),
                    Some(t) => t,
                };
                match t.kind {
                    TokenKind::FunctionIdentifier(id) => {
                        let mut args = Vec::new();
                        let mut last = None;
                        loop {
                            let next = self.next_token()?;
                            match next {
                                None => break,
                                Some(t)
                                    if t.kind == TokenKind::Comma
                                        && last != Some(TokenKind::Comma) =>
                                {
                                    last = Some(t.kind);
                                }
                                Some(t)
                                    if t.kind == TokenKind::Comma
                                        && last == Some(TokenKind::Comma) =>
                                {
                                    return Err(LexingError::ParseError("Unexpected ,".to_string()))
                                }
                                Some(t) => {
                                    let expr = self.next_expression(Some(t))?;
                                    args.push(expr);
                                }
                            }
                        }
                        Ok((id, args))
                    }
                    inv => Err(LexingError::ParseError(
                        format!("Expected FunctionIdentifier got {:?}", inv),
                    )),
                }
            }

            pub fn next_value(
                &mut self,
                v: Value<'lex>,
            ) -> Result<Option<Element<'lex>>, LexingError> {
                match self.next_token()? {
                    None => Ok(Some(Element::Expression(Expression::Value(v)))),
                    Some(s) => {
                        match s.kind {
                            TokenKind::Newline => {
                                Ok(Some(Element::Expression(Expression::Value(v))))
                            }
                            TokenKind::Semi => Ok(Some(Element::Expression(Expression::Value(v)))),
                            TokenKind::Value(v) => {
                                Err(LexingError::ParseError(format!("Unexpected value {}", v)))
                            }
                            TokenKind::BinOp(o) => {
                                let next = self.next_token()?;
                                let expr = self.next_expression(next)?;
                                Ok(Some(Element::Expression(Expression::BinExp(
                                    Box::new(Expression::Value(v)),
                                    o,
                                    Box::new(expr),
                                ))))
                            }
                            TokenKind::Minus => {
                                let next = self.next_token()?;
                                let expr = self.next_expression(next)?;
                                Ok(Some(Element::Expression(Expression::BinExp(
                                    Box::new(Expression::Value(v)),
                                    BinaryOperation::Sub,
                                    Box::new(expr),
                                ))))
                            }
                            TokenKind::Period => {
                                todo!() // value extension functions
                            }
                            k => Err(LexingError::ParseError(format!("Unexpected value {:?}", k))),
                        }
                    }
                }
            }

            pub fn next_args(&mut self) -> Result<(Vec<FunctionArgument<'lex>>, Option<Token>), LexingError> {
                let next = match self.next_token()? {
                    None => return Ok((Vec::new(), None)),
                    Some(t) => t
                };

                if next.kind != TokenKind::Lparen {
                    return Ok((Vec::new(), Some(next)))
                }

                let mut args = Vec::new();
                let mut last = None;
                let mut next_type = RigzType::Any;
                // fn foo(a, b, c: number = 1)
                loop {
                    let next = self.next_token()?;
                    match next {
                        None => return Err(LexingError::ParseError("Expected )".to_string())),
                        Some(t) if t.kind == TokenKind::Rparen => break,
                        Some(t) => {

                        }
                    }
                }

                Ok((args, None))
            }

            pub fn next_element(
                &mut self,
                token: Token<'lex>,
            ) -> Result<Option<Element<'lex>>, LexingError> {
                match token.kind {
                    TokenKind::Newline => Ok(None),
                    TokenKind::Value(v) => self.next_value(v),
                    TokenKind::Assign => Err(LexingError::ParseError("Unexpected =".to_string())),
                    TokenKind::BinOp(b) => {
                        Err(LexingError::ParseError(format!("Unexpected {:?}", b)))
                    }
                    TokenKind::Not => {
                        let token = self.next_token()?;
                        let next = self.next_expression(token)?;
                        Ok(Some(Element::Expression(Expression::UnaryExp(
                            UnaryOperation::Not,
                            Box::new(next),
                        ))))
                    }
                    TokenKind::Minus => {
                        let token = self.next_token()?;
                        let next = self.next_expression(token)?;
                        Ok(Some(Element::Expression(Expression::UnaryExp(
                            UnaryOperation::Neg,
                            Box::new(next),
                        ))))
                    }
                    TokenKind::Period => Err(LexingError::ParseError("Unexpected .".to_string())),
                    TokenKind::Comma => Err(LexingError::ParseError("Unexpected ,".to_string())),
                    TokenKind::FunctionDef => {
                        let next = match self.next_token()? {
                            None => return Err(LexingError::ParseError("Expected FunctionIdentifier".to_string())),
                            Some(s) => s
                        };
                        if let TokenKind::FunctionIdentifier(f) = next.kind {
                            // store function def
                            // list of statements
                            // expression as return
                            // end keyword, if no expression check last statement
                            let (arguments, nextToken) = self.next_args()?;
                            // check nextToken is : or end, otherwise treat as first statement/expression
                            let return_type = RigzType::Any;
                            let mut elements = Vec::new();
                            loop {
                                let next = self.next_token()?;
                                match next {
                                    None => return Err(LexingError::ParseError("Expected `end`".to_string())),
                                    Some(t) if t.kind == TokenKind::End => break,
                                    Some(t) => {
                                        let next = self.next_element(t)?;
                                        match next {
                                            None => return Err(LexingError::ParseError("Expected `end`".to_string())),
                                            Some(e) => elements.push(e)
                                        }
                                    }
                                }
                            }
                            let type_definition = FunctionDefinition { arguments, return_type };
                            self.function_declarations.insert(f, type_definition.clone());
                            Ok(Some(Element::Statement(Statement::FunctionDefinition { name: f, type_definition, elements })))
                        } else {
                            Err(LexingError::ParseError(format!("Expected FunctionIdentifier received {:?}", next.kind)))
                        }
                    }
                    TokenKind::Identifier(id) => self.handle_identifier(id),
                    TokenKind::FunctionIdentifier(id) => {
                        // or module contains function
                        if self.function_declarations.contains_key(id) {
                            let mut args = Vec::new();
                            let mut last = None;
                            loop {
                                let next = self.next_token()?;
                                match next {
                                    None => break,
                                    Some(t)
                                        if t.kind == TokenKind::Comma
                                            && last != Some(TokenKind::Comma) =>
                                    {
                                        last = Some(t.kind);
                                    }
                                    Some(t)
                                        if t.kind == TokenKind::Comma
                                            && last == Some(TokenKind::Comma) =>
                                    {
                                        return Err(LexingError::ParseError(
                                            "Unexpected ,".to_string(),
                                        ))
                                    }
                                    Some(t) => {
                                        let expr = self.next_expression(Some(t))?;
                                        args.push(expr);
                                    }
                                }
                            }
                            Ok(Some(Element::Expression(Expression::FunctionCall(
                                id, args,
                            ))))
                        } else {
                            self.handle_identifier(id)
                        }
                    }
                    TokenKind::Lparen => {
                        todo!() // create scope
                    }
                    TokenKind::Rparen => Err(LexingError::ParseError("Unexpected )".to_string())),
                    TokenKind::Lcurly => {
                        let map = IndexMap::new();
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
                                                    return Err(LexingError::ParseError(
                                                        "Unexpected ,".to_string(),
                                                    ));
                                                }
                                                self.current_token = Some(t)
                                            }
                                        }
                                        continue;
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
                        // allow do |v| end
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
                    Some(t) => Ok(Some(t)),
                }
            }

            fn load_value(&mut self, value: Value<'lex>) {
                let next = self.next_register();
                self.builder.add_load_instruction(next, value);
            }

            fn to_value(&mut self, expressions: Vec<Expression<'lex>>) -> Value<'lex> {
                match expressions.len() {
                    0 => Value::None,
                    1 => match &expressions[0] {
                        Expression::Value(v) => v.clone(),
                        _ => todo!(),
                    },
                    _ => todo!(),
                }
            }

            fn build_expression(&mut self, expression: Expression<'lex>) -> Result<(), LexingError>{
                match expression {
                    Expression::Value(v) => self.load_value(v),
                    Expression::List(_) => {}
                    Expression::Map(_) => {}
                    Expression::Identifier(i) => {
                        let next = self.next_register();
                        self.builder.add_get_variable_instruction(i, next);
                    }
                    Expression::BinExp(lhs, op, rhs) => {
                        self.build_expression(*lhs)?;
                        let lhs = self.last;
                        self.build_expression(*rhs)?;
                        let rhs = self.last;
                        let next = self.next_register();
                        self.builder.add_instruction(Instruction::Binary(Binary {
                            op,
                            lhs,
                            rhs,
                            output: next,
                        }));
                    }
                    Expression::UnaryExp(op, expression) => {
                        self.build_expression(*expression)?;
                        let from = self.last;
                        let output = self.next_register();
                        self.builder.add_instruction(Instruction::Unary(Unary {
                            op,
                            from,
                            output,
                        }));
                    }
                    Expression::FunctionCall(name, def) => match name {
                        "puts" => {
                            let list = self.to_value(def);
                            self.load_value(list);
                            self.builder.add_puts_instruction(vec![self.last]);
                            self.set_last(0);
                        }
                        "eprintln" => {
                            let list = self.to_value(def);
                            self.load_value(list);
                            self.builder.add_eprintln_instruction(self.last, 0);
                            self.set_last(0);
                        }
                        "eprint" => {
                            let list = self.to_value(def);
                            self.load_value(list);
                            self.builder.add_eprint_instruction(self.last, 0);
                            self.set_last(0);
                        }
                        "println" => {
                            let list = self.to_value(def);
                            self.load_value(list);
                            self.builder.add_println_instruction(self.last, 0);
                            self.set_last(0);
                        }
                        "print" => {
                            let list = self.to_value(def);
                            self.load_value(list);
                            self.builder.add_print_instruction(self.last, 0);
                            self.set_last(0);
                        }
                        // todo logging
                        fun => {
                            let f = match self.function_declarations.get(fun) {
                                None => return Err(LexingError::ParseError(format!("Unknown function {}", fun))),
                                Some(f) => f.clone(),
                            };
                            let s = match self.function_scopes.get(fun) {
                                None => return Err(LexingError::ParseError(format!("function {} has not been stored in VM", fun))),
                                Some(f) => f.clone()
                            };
                            let mut arg_index = 0;
                            for arg in &s.args {
                                let expression = &def[arg_index];
                                match f.arguments[arg_index].name {
                                    None => {
                                        match expression {
                                            Expression::Value(v) => {
                                                self.builder.add_load_instruction(*arg, v.clone());
                                            }
                                            _ => todo!()
                                        }
                                    }
                                    Some(n) => {
                                        self.build_expression(expression.clone())?;
                                        self.builder.add_load_let_instruction(n, self.last);
                                    }
                                };
                                arg_index += 1;
                            }
                            self.builder.add_call_instruction(s.scope, s.output);
                        }
                    },
                    Expression::Scope(s) => {
                        self.builder.enter_scope();
                        for expr in s {
                            self.build_expression(expr)?;
                        }
                        self.builder.exit_scope(self.last);
                    }
                    Expression::ModuleCall(m, f, args) => {
                        let mut reg_args = Vec::with_capacity(args.len());
                        for arg in args {
                            self.build_expression(arg)?;
                            reg_args.push(self.last);
                        }
                        let output = self.next_register();
                        if m == "__VM" {
                            self.builder
                                .add_call_vm_extension_module_instruction(m, f, reg_args, output);
                        } else {
                            self.builder
                                .add_call_module_instruction(m, f, reg_args, output);
                        }
                    }
                }
                Ok(())
            }

            fn build_assignment_value(
                &mut self,
                name: &'lex str,
                mutable: bool,
                value: Value<'lex>,
            ) {
                self.load_value(value);
                if mutable {
                    self.builder.add_load_mut_instruction(name, self.last);
                } else {
                    self.builder.add_load_let_instruction(name, self.last);
                }
            }

            fn build_assignment_identifier(
                &mut self,
                name: &'lex str,
                mutable: bool,
                id: &'lex str,
            ) {
                let next = self.next_register();
                self.builder.add_get_variable_instruction(id, next);
                if mutable {
                    self.builder.add_load_mut_instruction(name, self.last);
                } else {
                    self.builder.add_load_let_instruction(name, self.last);
                }
            }

            fn build_statement(&mut self, statement: Statement<'lex>) -> Result<(), LexingError> {
                match statement {
                    Statement::Assignment {
                        name,
                        mutable,
                        expression,
                    } => {
                        match expression {
                            Expression::Value(v) => self.build_assignment_value(name, mutable, v),
                            Expression::Identifier(id) => {
                                self.build_assignment_identifier(name, mutable, id)
                            }
                            Expression::List(_) => {
                                // if list<value> store as value, otherwise create scope
                            }
                            Expression::Map(_) => {
                                // if list<value> store as value, otherwise create scope
                            }
                            Expression::BinExp(lhs, op, rhs) => {
                                self.builder.enter_scope();
                                let scope = self.builder.sp;
                                self.build_expression(*lhs)?;
                                let lhs = self.last;
                                self.build_expression(*rhs)?;
                                let rhs = self.last;
                                let output = self.next_register();
                                self.builder.add_instruction(Instruction::Binary(Binary {
                                    op,
                                    lhs,
                                    rhs,
                                    output,
                                }));
                                self.builder.exit_scope(output);
                                self.load_value(Value::ScopeId(scope, output));
                                self.builder.add_load_let_instruction(name, self.last);
                            }
                            Expression::UnaryExp(op, expr) => {
                                self.builder.enter_scope();
                                self.build_expression(*expr)?;
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
                            Expression::ModuleCall(_, _, _) => todo!(),
                        }
                    }
                    Statement::FunctionDefinition { name, type_definition, elements } => {
                        self.build_function_definition(name, type_definition, elements)?;
                    }
                    Statement::Expression(e) => self.build_expression(e)?,
                }
                Ok(())
            }

    fn build_function_definition(&mut self, name: &'lex str, type_definition: FunctionDefinition<'lex>, elements: Vec<Element<'lex>>) -> Result<(), LexingError> {
        let mut args = Vec::with_capacity(type_definition.arguments.len());
        for _arg in type_definition.arguments {
            args.push(self.next_register());
        }
        self.builder.enter_scope();
        let scope = self.builder.sp;
        for e in elements {
            self.build_element(e)?;
        }
        let output = self.last;
        self.builder.exit_scope(output);

        let fd = FunctionCallDefinition {
            scope,
            args,
            output,
        };
        self.function_scopes.insert(name, fd);
        Ok(())
    }

    fn build_element(&mut self, element: Element<'lex>) -> Result<(), LexingError> {
                match element {
                    Element::Statement(s) => self.build_statement(s),
                    Element::Expression(e) => self.build_expression(e),
                }
            }
        }
//     };
// }
//
// gen_parser!(Parser, VMBuilder, VMBuilder::new);
// gen_parser!(VMParser, VM, VM::new);