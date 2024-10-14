use crate::ast::{Element, Expression, Scope, Statement};
use crate::modules::json::JsonModule;
use crate::modules::std_lib::StdLibModule;
use crate::modules::vm::VMModule;
use rigz_vm::{Module, Register, VMBuilder, Value, VM};

pub(crate) struct ProgramParser<'vm> {
    builder: VMBuilder<'vm>,
    current: Register,
    last: Register,
}

fn add_default_modules(builder: &mut VMBuilder) {
    builder.register_module(VMModule {});
    builder.register_module(StdLibModule {});
    builder.register_module(JsonModule {});
}

impl<'vm> Default for ProgramParser<'vm> {
    fn default() -> Self {
        let mut builder = VMBuilder::new();
        ProgramParser {
            builder,
            current: 2, // 0 & 1 are reserved
            last: 0,
        }
    }
}

impl<'vm> ProgramParser<'vm> {
    pub(crate) fn new() -> Self {
        let mut p = ProgramParser::default();
        add_default_modules(&mut p.builder);
        p
    }

    // Does not include default modules
    pub(crate) fn with_modules(modules: Vec<impl Module<'vm> + 'static>) -> Self {
        let mut p = ProgramParser::default();
        // todo program parser needs to evaluate m.trait_definition() to store available functions
        for m in modules {
            p.builder.register_module(m);
        }
        p
    }

    pub(crate) fn parse_element(&mut self, element: Element<'vm>) {
        match element {
            Element::Statement(s) => self.parse_statement(s),
            Element::Expression(e) => self.parse_expression(e),
        }
    }

    pub(crate) fn parse_statement(&mut self, statement: Statement<'vm>) {
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
            // Statement::Return(e) => match e {
            //     None => {
            //         self.builder.add_ret_instruction(0);
            //     }
            //     Some(_) => {}
            // },
            Statement::FunctionDefinition {
                name,
                type_definition,
                body,
            } => {
                todo!()
            }
        }
    }

    pub(crate) fn parse_expression(&mut self, expression: Expression<'vm>) {
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
                let truthy = self.parse_scope(then);
                match branch {
                    None => {
                        let output = self.next_register();
                        self.builder.add_if_instruction(cond, truthy, output);
                    }
                    Some(p) => {
                        let falsy = self.parse_scope(p);
                        let output = self.next_register();
                        self.builder
                            .add_if_else_instruction(cond, truthy, falsy, output);
                    }
                }
            }
            Expression::Unless { condition, then } => {
                self.parse_expression(*condition);
                let cond = self.last;
                let unless = self.parse_scope(then);
                let output = self.next_register();
                self.builder.add_unless_instruction(cond, unless, output);
            }
            Expression::List(_) => {
                // store static part of list first, values only, then modify
            }
            Expression::Map(_) => {
                // store static part of map first, values only, then modify
            }
            Expression::FunctionCall(_, _) => {
                todo!()
            }
            Expression::InstanceFunctionCall(_, _, _) => {
                todo!()
            }
            Expression::Scope(s) => {
                let _ = self.parse_scope(s);
            }
            Expression::Cast(e, t) => {
                self.parse_expression(*e);
                let output = self.next_register();
                self.builder.add_cast_instruction(self.last, t, output);
            }
            Expression::Symbol(s) => {
                let next = self.next_register();
                // todo create a symbols cache
                self.builder
                    .add_load_instruction(next, Value::String(s.to_string()));
            }
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

    fn parse_scope(&mut self, scope: Scope<'vm>) -> usize {
        self.builder.enter_scope();
        let res = self.builder.sp;
        for e in scope.elements {
            self.parse_element(e);
        }
        let next = self.next_register();
        self.builder.exit_scope(next);
        res
    }

    pub(crate) fn build(mut self) -> VM<'vm> {
        self.builder.add_halt_instruction(self.last);
        self.builder.build()
    }
}
