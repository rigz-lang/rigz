use crate::ast::{
    Element, Expression, FunctionDeclaration, ModuleTraitDefinition, Parser, Scope, Statement,
    TraitDefinition,
};
use crate::modules::{JsonModule, StdLibModule, VMModule};
use crate::{FunctionDefinition, FunctionSignature, FunctionType};
use indexmap::map::Entry;
use indexmap::IndexMap;
use log::warn;
use rigz_vm::{Clear, Module, Register, RegisterValue, RigzType, VMBuilder, VMError, Value, VM};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CallSite<'vm> {
    Scope(usize),
    Module(&'vm str),
}

pub(crate) struct ProgramParser<'vm> {
    builder: VMBuilder<'vm>,
    current: Register,
    last: Register,
    modules: HashMap<&'vm str, ModuleTraitDefinition<'vm>>,
    function_scopes: IndexMap<&'vm str, Vec<(FunctionSignature<'vm>, CallSite<'vm>)>>,
}

impl<'vm> Default for ProgramParser<'vm> {
    fn default() -> Self {
        let mut builder = VMBuilder::new();
        ProgramParser {
            builder,
            current: 0,
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
            self.parse_trait_definition_for_module(name, module.definition);
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
            Statement::FunctionDefinition(fd) => {
                self.parse_function_definition(fd);
            }
        }
    }

    pub(crate) fn parse_function_definition(&mut self, function_definition: FunctionDefinition<'vm>) {
        let FunctionDefinition {
            name,
            type_definition,
            body,
        } = function_definition;
        let f_def = self.parse_scope(body);
        // todo convert type definition into function call (reserve registers & map args)
        match self.function_scopes.entry(name) {
            Entry::Occupied(mut entry) => {
                entry
                    .get_mut()
                    .push((type_definition, CallSite::Scope(f_def)));
            }
            Entry::Vacant(e) => {
                e.insert(vec![(type_definition, CallSite::Scope(f_def))]);
            }
        }
    }

    pub(crate) fn parse_trait_definition_for_module(&mut self, module_name: &'static str, trait_definition: TraitDefinition<'vm>) {
        for func in trait_definition.functions {
            match func {
                FunctionDeclaration::Declaration { type_definition, name } => {
                    match self.function_scopes.entry(name) {
                        Entry::Occupied(mut entry) => {
                            entry
                                .get_mut()
                                .push((type_definition, CallSite::Module(module_name)));
                        }
                        Entry::Vacant(e) => {
                            e.insert(vec![(type_definition, CallSite::Module(module_name))]);
                        }
                    }
                }
                FunctionDeclaration::Definition(fd) => {
                    self.parse_function_definition(fd)
                }
            }
        }
    }

    pub(crate) fn parse_trait_definition(&mut self, trait_definition: TraitDefinition<'vm>) {

    }

    pub(crate) fn parse_expression(&mut self, expression: Expression<'vm>) {
        match expression {
            Expression::Value(v) => self.parse_value(v),
            Expression::BinExp(a, op, b) => {
                let (a, b, clear) = match (*a, *b) {
                    (Expression::Value(a), Expression::Value(b)) => {
                        self.parse_value(a);
                        let a = self.last;
                        self.parse_value(b);
                        let b = self.last;
                        (a, b, Clear::Two(a, b))
                    }
                    (a, Expression::Value(b)) => {
                        self.parse_expression(a);
                        let a = self.last;
                        self.parse_value(b);
                        let b = self.last;
                        (a, b, Clear::One(b))
                    }
                    (Expression::Value(a), b) => {
                        self.parse_value(a);
                        let a = self.last;
                        self.parse_expression(b);
                        let b = self.last;
                        (a, b, Clear::One(a))
                    }
                    (a, b) => {
                        self.parse_expression(a);
                        let a = self.last;
                        self.parse_expression(b);
                        let b = self.last;
                        let next = self.next_register();
                        self.builder.add_binary_instruction(op, a, b, next);
                        return;
                    }
                };
                let next = self.next_register();
                self.builder
                    .add_binary_clear_instruction(op, a, b, clear, next);
            }
            Expression::UnaryExp(op, ex) => match *ex {
                Expression::Value(v) => {
                    self.parse_value(v);
                    let r = self.last;
                    let next = self.next_register();
                    self.builder.add_unary_clear_instruction(op, r, next);
                }
                ex => {
                    self.parse_expression(ex);
                    let r = self.last;
                    let next = self.next_register();
                    self.builder.add_unary_instruction(op, r, next);
                }
            },
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
                self.builder
                    .add_load_instruction(r, Value::List(base).into());
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
                self.builder
                    .add_load_instruction(r, Value::Map(base).into());
                if !remaining.is_empty() {
                    todo!("expressions in map not supported yet")
                }
                // store static part of map first, values only, then modify
            }
            // todo use clear in function calls when appropriate
            Expression::FunctionCall(name, args) => match self.function_scopes.get(name) {
                None => {
                    let next = self.next_register();
                    self.builder.add_load_instruction(
                        next,
                        Value::Error(VMError::InvalidModuleFunction(format!(
                            "Function {name} does not exist"
                        )))
                        .into(),
                    );
                }
                Some(f) => {
                    todo!("Find best match for {:?}", f)
                }
            },
            Expression::TypeFunctionCall(rigz_type, name, args) => {
                match rigz_type {
                    RigzType::VM => {
                        let mut call_args = Vec::with_capacity(args.len());
                        for a in args {
                            self.parse_expression(a);
                            call_args.push(self.last);
                        }
                        let output = self.next_register();
                        // todo this really should check function definitions before adding the instruction (require mut VM?)
                        self.builder.add_call_vm_extension_module_instruction(
                            "VM", name, call_args, output,
                        );
                    }
                    rt => {
                        todo!("Support all rigz types as function calls {rt}")
                    }
                }
            }
            Expression::InstanceFunctionCall(exp, calls, args) => {
                let name = match calls.last() {
                    None => {
                        let next = self.next_register();
                        self.builder.add_load_instruction(
                            next,
                            Value::Error(VMError::InvalidModuleFunction(format!(
                                "Invalid Instance call for {:?}",
                                *exp
                            )))
                            .into(),
                        );
                        return;
                    }
                    Some(s) => *s,
                };
                let f = match self.function_scopes.get(name) {
                    None => {
                        let next = self.next_register();
                        self.builder.add_load_instruction(
                            next,
                            Value::Error(VMError::InvalidModuleFunction(format!(
                                "Extension Function {name} does not exist"
                            )))
                            .into(),
                        );
                        return;
                    }
                    Some(f) => f.clone()
                };
                if f.len() == 1 {
                    let (sig, call) = f[0].clone();
                    let mut call_args = Vec::with_capacity(args.len());
                    for a in args {
                        self.parse_expression(a);
                        call_args.push(self.last);
                    }
                    match call {
                        CallSite::Scope(_) => {
                            todo!("Support scope functions {:?}", f)
                        }
                        CallSite::Module(m) => {
                            match sig.self_type {
                                None => {
                                    let output = self.next_register();
                                    self.builder.add_call_module_instruction(m, name, call_args, output);
                                }
                                Some(t) => {
                                    match t.rigz_type {
                                        RigzType::VM => {
                                            let output = self.next_register();
                                            self.builder.add_call_vm_extension_module_instruction(m, name, call_args, output);
                                        }
                                        _ if t.mutable => {
                                            self.parse_expression(*exp);
                                            let this = self.last;
                                            self.builder.add_call_mutable_extension_module_instruction(m, name, this, call_args);
                                        }
                                        _ => {
                                            self.parse_expression(*exp);
                                            let this = self.last;
                                            let output = self.next_register();
                                            self.builder.add_call_extension_module_instruction(m, name, this, call_args, output);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    todo!("Find best match for extension {:?}", f)
                }
            }
            Expression::Scope(s) => {
                let s = self.parse_scope(s);
                let next = self.next_register();
                let output = self.next_register();
                self.builder
                    .add_load_instruction(next, RegisterValue::ScopeId(s, output));
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
                    .add_load_instruction(next, Value::String(s.to_string()).into());
            }
        }
    }

    fn next_register(&mut self) -> Register {
        self.last = self.current;
        self.current += 1;
        self.last
    }

    fn parse_value(&mut self, value: Value) {
        self.builder
            .add_load_instruction(self.current, value.into());
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
