use crate::ast::{Element, Expression, Program, Statement};
use crate::runtime::RuntimeError;
use rigz_vm::{Register, VMBuilder, Value, VM};

struct ProgramParser<'vm> {
    builder: VMBuilder<'vm>,
    current: Register,
    last: Register,
}

impl<'vm> Default for ProgramParser<'vm> {
    fn default() -> Self {
        ProgramParser {
            builder: Default::default(),
            current: 2, // 0 & 1 are reserved
            last: 0,
        }
    }
}

impl<'vm> ProgramParser<'vm> {
    fn new() -> Self {
        ProgramParser::default()
    }

    fn parse_statement(&mut self, statement: Statement<'vm>) {
        match statement {
            Statement::Assignment {
                name,
                mutable,
                expression,
            } => {
                self.parse_expression(expression);
                if mutable {
                    self.builder.add_load_mut_instruction(name, self.last);
                } else {
                    self.builder.add_load_let_instruction(name, self.last);
                }
            }
            Statement::Return(e) => match e {
                None => {
                    self.builder.add_ret_instruction(0);
                }
                Some(_) => {}
            },
            _ => todo!(),
        }
    }

    fn parse_expression(&mut self, expression: Expression<'vm>) {
        match expression {
            Expression::Value(v) => self.parse_value(v),
            // TODO use clear when appropriate
            Expression::BinExp(a, op, b) => {
                self.parse_expression(*a);
                let a = self.last;
                self.parse_expression(*b);
                let b = self.last;
                let next = self.next_register();
                self.builder.add_binary_instruction(op, a, b, next);
            }
            Expression::UnaryExp(op, ex) => {
                self.parse_expression(*ex);
                let r = self.last;
                let next = self.next_register();
                self.builder.add_unary_instruction(op, r, next);
            }
            Expression::Identifier(id) => {
                let next = self.next_register();
                self.builder.add_get_variable_instruction(id, next);
            }
            Expression::If {
                condition,
                then,
                branch,
            } => {
                self.parse_expression(*condition);
                let cond = self.last;
                let truthy = self.parse_program(then);
                match branch {
                    None => {
                        let output = self.next_register();
                        self.builder.add_if_instruction(cond, truthy, output);
                    }
                    Some(p) => {
                        let falsy = self.parse_program(p);
                        let output = self.next_register();
                        self.builder
                            .add_if_else_instruction(cond, truthy, falsy, output);
                    }
                }
            }
            Expression::Unless { condition, then } => {
                self.parse_expression(*condition);
                let cond = self.last;
                let unless = self.parse_program(then);
                let output = self.next_register();
                self.builder.add_unless_instruction(cond, unless, output);
            }
            _ => todo!(),
        }
    }

    fn next_register(&mut self) -> Register {
        self.last = self.current;
        self.current += 1;
        self.last
    }

    fn parse_value(&mut self, value: Value) {
        self.builder.add_load_instruction(self.current, value);
        self.next_register();
    }

    fn parse_program(&mut self, program: Program<'vm>) -> usize {
        todo!()
    }

    fn build(mut self) -> VM<'vm> {
        self.builder.add_halt_instruction(self.last);
        self.builder.build()
    }
}

impl<'l> TryInto<VM<'l>> for Program<'l> {
    type Error = RuntimeError;

    fn try_into(self) -> Result<VM<'l>, Self::Error> {
        self.validate().map_err(|e| e.into())?;

        let mut builder = ProgramParser::new();
        for element in self.elements {
            match element {
                Element::Statement(s) => builder.parse_statement(s),
                Element::Expression(e) => builder.parse_expression(e),
            }
        }
        Ok(builder.build())
    }
}
