use crate::ast::{
    Element, Expression, FunctionDeclaration, ModuleTraitDefinition, Parser, Scope, Statement,
    TraitDefinition,
};
use crate::modules::{JsonModule, StdLibModule, VMModule};
use crate::FunctionDefinition;
use indexmap::IndexMap;
use log::warn;
use rigz_vm::{Module, Register, VMBuilder, Value, VM};
use std::collections::HashMap;

pub(crate) struct ProgramParser<'vm> {
    builder: VMBuilder<'vm>,
    current: Register,
    last: Register,
    modules: HashMap<&'vm str, ModuleTraitDefinition<'vm>>,
    function_scopes: IndexMap<&'vm str, Vec<(FunctionDefinition<'vm>, usize)>>,
}

impl<'vm> Default for ProgramParser<'vm> {
    fn default() -> Self {
        let mut builder = VMBuilder::new();
        ProgramParser {
            builder,
            current: 2, // 0 & 1 are reserved
            last: 0,
            modules: HashMap::new(),
            function_scopes: IndexMap::new(),
        }
    }
}

impl<'vm> ProgramParser<'vm> {
    pub(crate) fn new() -> Self {
        let mut p = ProgramParser::default();
        p.add_default_modules();
        p
    }

    fn add_default_modules(&mut self) {
        self.register_module(VMModule {});
        self.register_module(StdLibModule {});
        self.register_module(JsonModule {});
    }

    fn register_module(&mut self, module: impl Module<'vm> + 'static) {
        let def = module.trait_definition();
        self.parse_module_trait_definition(module.name(), def);
        self.builder.register_module(module);
    }

    fn parse_module_trait_definition(&mut self, name: &'static str, def: &'static str) {
        let mut p = match Parser::prepare(def) {
            Ok(p) => p,
            Err(e) => panic!("Failed to read {} module definition: {e}", name),
        };
        let module = match p.parse_module_trait_definition() {
            Ok(d) => d,
            Err(e) => panic!("Failed to parse {} module definition: {e}", name),
        };

        if p.has_tokens() {
            warn!("leftover tokens after parsing {} module definition", name);
        }

        if module.definition.functions.is_empty() {
            warn!("empty function definitions for module {name}");
        }

        if name != module.definition.name {
            warn!(
                "mismatched name for module {name} != {}, using {name}",
                module.definition.name
            );
        }

        if module.imported {
            self.parse_trait_definition(module.definition);
            // trait definition is useless after import
            self.modules.insert(
                name,
                ModuleTraitDefinition {
                    imported: true,
                    definition: TraitDefinition {
                        name,
                        functions: vec![],
                    },
                },
            );
        } else {
            self.modules.insert(module.definition.name, module);
        }
    }

    // Does not include default modules
    pub(crate) fn with_modules(modules: Vec<impl Module<'vm> + 'static>) -> Self {
        let mut p = ProgramParser::default();
        // todo program parser needs to evaluate m.trait_definition() to store available functions
        for m in modules {
            p.register_module(m);
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
            Statement::Trait(t) => {
                self.parse_trait_definition(t);
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

    pub(crate) fn parse_trait_definition(&mut self, trait_definition: TraitDefinition<'vm>) {}

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
            Expression::List(list) => {
                let mut base = Vec::new();
                let mut remaining = Vec::new();
                for v in list {
                    match v {
                        Expression::Value(v) => {
                            base.push(v);
                        }
                        v => {
                            remaining.push(v);
                        }
                    }
                }
                let r = self.next_register();
                self.builder.add_load_instruction(r, Value::List(base));
                if !remaining.is_empty() {
                    todo!("expressions in list not supported yet")
                }
            }
            Expression::Map(map) => {
                let mut base = IndexMap::new();
                let mut remaining = Vec::new();
                for (k, v) in map {
                    match (k, v) {
                        (Expression::Value(k), Expression::Value(v)) => {
                            base.insert(k, v);
                        }
                        (Expression::Identifier(k), Expression::Value(v)) => {
                            base.insert(Value::String(k.to_string()), v);
                        }
                        (k, v) => {
                            remaining.push((k, v));
                        }
                    }
                }
                let r = self.next_register();
                self.builder.add_load_instruction(r, Value::Map(base));
                if !remaining.is_empty() {
                    todo!("expressions in map not supported yet")
                }
                // store static part of map first, values only, then modify
            }
            Expression::FunctionCall(name, args) => {
                todo!()
            }
            Expression::InstanceFunctionCall(exp, calls, args) => {
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
