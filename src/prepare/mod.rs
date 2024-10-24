use crate::ast::{
    Assign, Element, Exposed, Expression, FunctionDeclaration, ModuleTraitDefinition, Parser,
    Scope, Statement, TraitDefinition, ValidationError,
};
use crate::modules::{JsonModule, StdLibModule, VMModule};
use crate::{FunctionArgument, FunctionDefinition, FunctionSignature, FunctionType};
use indexmap::map::Entry;
use indexmap::{IndexMap, IndexSet};
use log::warn;
use rigz_vm::{
    Clear, Module, Number, Register, RegisterValue, RigzType, VMBuilder, VMError, Value, VM,
};
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum CallSite<'vm> {
    Scope(usize, Register),
    Module(&'vm str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallSignature<'vm> {
    // todo how do var args work with the VM? push to stack then load to argument list?
    pub arguments: Vec<(FunctionArgument<'vm>, Register)>,
    pub return_type: (FunctionType, Register),
    pub self_type: Option<(FunctionType, Register)>,
    pub positional: bool,
}

type FunctionCallSignatures<'vm> = Vec<(FunctionCallSignature<'vm>, CallSite<'vm>)>;

pub(crate) struct ProgramParser<'vm> {
    builder: VMBuilder<'vm>,
    current: Register,
    last: Register,
    modules: HashMap<&'vm str, ModuleTraitDefinition<'vm>>,
    // todo nested functions are global, they should be removed if invalid
    function_scopes: IndexMap<&'vm str, FunctionCallSignatures<'vm>>,
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
        self.register_module(VMModule {})
            .expect("Failed to register VMModule");
        self.register_module(StdLibModule {})
            .expect("Failed to register StdLibModule");
        self.register_module(JsonModule {})
            .expect("Failed to register JsonModule");
    }

    fn register_module(
        &mut self,
        module: impl Module<'vm> + 'static,
    ) -> Result<(), ValidationError> {
        let def = module.trait_definition();
        self.parse_module_trait_definition(module.name(), def)?;
        self.builder.register_module(module);
        Ok(())
    }

    fn parse_module_trait_definition(
        &mut self,
        name: &'static str,
        def: &'static str,
    ) -> Result<(), ValidationError> {
        let mut p = match Parser::prepare(def) {
            Ok(p) => p,
            Err(e) => {
                return Err(ValidationError::ModuleError(format!(
                    "Failed to read {} module definition: {e}",
                    name
                )))
            }
        };
        let module = match p.parse_module_trait_definition() {
            Ok(d) => d,
            Err(e) => {
                return Err(ValidationError::ModuleError(format!(
                    "Failed to parse {} module definition: {e}",
                    name
                )))
            }
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
            self.parse_trait_definition_for_module(name, module.definition)?;
            // trait definition is useless after import
            self.modules
                .insert(name, ModuleTraitDefinition::imported(name));
        } else {
            self.modules.insert(name, module);
        }
        Ok(())
    }

    // Does not include default modules
    pub(crate) fn with_modules(
        modules: Vec<impl Module<'vm> + 'static>,
    ) -> Result<Self, ValidationError> {
        let mut p = ProgramParser::default();
        for m in modules {
            p.register_module(m)?;
        }
        Ok(p)
    }

    pub(crate) fn parse_element(&mut self, element: Element<'vm>) -> Result<(), ValidationError> {
        match element {
            Element::Statement(s) => self.parse_statement(s),
            Element::Expression(e) => self.parse_expression(e),
        }
    }

    pub(crate) fn parse_statement(
        &mut self,
        statement: Statement<'vm>,
    ) -> Result<(), ValidationError> {
        match statement {
            Statement::Assignment {
                lhs: Assign::Identifier(name, mutable),
                expression,
            } => {
                self.parse_expression(expression)?;
                if mutable {
                    self.builder.add_load_mut_instruction(name, self.last);
                } else {
                    self.builder.add_load_let_instruction(name, self.last);
                }
            }
            Statement::Assignment {
                lhs: Assign::This,
                expression,
            } => {
                let this = self.mutable_this();
                self.parse_expression(expression)?;
                let rhs = self.last;
                self.builder
                    .add_load_instruction(this, RegisterValue::Register(rhs));
            }
            Statement::BinaryAssignment {
                lhs: Assign::Identifier(name, _),
                op,
                expression,
            } => {
                let this = self.next_register();
                self.builder
                    .add_get_mutable_variable_instruction(name, this);
                self.parse_expression(expression)?;
                let rhs = self.last;
                self.builder.add_binary_assign_instruction(op, this, rhs);
            }
            Statement::Import(name) => {
                self.parse_import(name)?;
            }
            Statement::Export(exposed) => {
                return Err(ValidationError::InvalidExport(format!(
                    "Exports are currently not supported {exposed:?}"
                )))
            }
            Statement::BinaryAssignment {
                lhs: Assign::This,
                op,
                expression,
            } => {
                let this = self.mutable_this();
                self.parse_expression(expression)?;
                let rhs = self.last;
                self.builder.add_binary_assign_instruction(op, this, rhs);
            }
            Statement::Trait(t) => {
                self.parse_trait_definition(t)?;
            }
            // Statement::Return(e) => match e {
            //     None => {
            //         self.builder.add_ret_instruction(0);
            //     }
            //     Some(_) => {}
            // },
            Statement::FunctionDefinition(fd) => {
                self.parse_function_definition(fd)?;
            }
        }
        Ok(())
    }

    fn mutable_this(&mut self) -> Register {
        let this = self.next_register();
        self.builder.add_get_self_instruction(this, true);
        this
    }

    fn this(&mut self) -> Register {
        let this = self.next_register();
        self.builder.add_get_self_instruction(this, false);
        this
    }

    pub(crate) fn parse_function_definition(
        &mut self,
        function_definition: FunctionDefinition<'vm>,
    ) -> Result<(), ValidationError> {
        let FunctionDefinition {
            name,
            type_definition,
            body,
        } = function_definition;
        let type_definition = self.parse_type_signature(type_definition)?;
        let current_scope = self.builder.sp;
        self.builder.enter_scope();
        for (arg, reg) in &type_definition.arguments {
            // todo handle varargs
            if arg.function_type.mutable {
                self.builder.add_load_mut_instruction(arg.name, *reg);
            } else {
                self.builder.add_load_let_instruction(arg.name, *reg);
            }
        }
        // todo store arguments variable
        let f_def = self.builder.sp;
        let (_, output) = type_definition.return_type;
        let self_type = type_definition.self_type.clone();
        match self.function_scopes.entry(name) {
            Entry::Occupied(mut entry) => {
                entry
                    .get_mut()
                    .push((type_definition, CallSite::Scope(f_def, output)));
            }
            Entry::Vacant(e) => {
                e.insert(vec![(type_definition, CallSite::Scope(f_def, output))]);
            }
        }
        for e in body.elements {
            match e {
                Element::Expression(Expression::This) => match &self_type {
                    Some(t) if t.0.mutable => {
                        self.mutable_this();
                    }
                    _ => self.parse_element(e)?,
                },
                e => self.parse_element(e)?,
            }
        }
        self.builder
            .add_load_instruction(output, RegisterValue::Register(self.last));
        self.builder.exit_scope(current_scope, output);
        Ok(())
    }

    pub(crate) fn parse_trait_definition_for_module(
        &mut self,
        module_name: &'vm str,
        trait_definition: TraitDefinition<'vm>,
    ) -> Result<(), ValidationError> {
        for func in trait_definition.functions {
            match func {
                FunctionDeclaration::Declaration {
                    type_definition,
                    name,
                } => {
                    let type_definition = self.parse_type_signature(type_definition)?;
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
                FunctionDeclaration::Definition(fd) => self.parse_function_definition(fd)?,
            }
        }
        Ok(())
    }

    pub(crate) fn parse_type_signature(
        &mut self,
        function_signature: FunctionSignature<'vm>,
    ) -> Result<FunctionCallSignature<'vm>, ValidationError> {
        let FunctionSignature {
            arguments,
            return_type,
            self_type,
            positional,
        } = function_signature;
        let self_type = match self_type {
            None => None,
            Some(f) => Some((f, self.next_register())),
        };

        let mut args = Vec::with_capacity(arguments.len());

        for arg in arguments {
            args.push((arg, self.next_register()))
        }

        let return_type = if return_type.mutable {
            let this = match self_type {
                None => {
                    return Err(ValidationError::InvalidFunction(
                        "Cannot have mutable return type on non-extension function".to_string(),
                    ))
                }
                Some((_, t)) => t,
            };
            (return_type, this)
        } else {
            (return_type, self.next_register())
        };

        Ok(FunctionCallSignature {
            arguments: args,
            return_type,
            self_type,
            positional,
        })
    }

    pub(crate) fn parse_trait_definition(
        &mut self,
        trait_definition: TraitDefinition<'vm>,
    ) -> Result<(), ValidationError> {
        for func in trait_definition.functions {
            match func {
                FunctionDeclaration::Declaration {
                    type_definition,
                    name,
                } => {
                    todo!("Support Function Declarations in Trait definition, call site is determined by impl")
                }
                FunctionDeclaration::Definition(fd) => self.parse_function_definition(fd)?,
            }
        }
        Ok(())
    }

    pub(crate) fn parse_expression(
        &mut self,
        expression: Expression<'vm>,
    ) -> Result<(), ValidationError> {
        match expression {
            Expression::This => {
                let _ = self.this();
            }
            // Expression::Index(_, _) => {}
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
                        self.parse_expression(a)?;
                        let a = self.last;
                        self.parse_value(b);
                        let b = self.last;
                        (a, b, Clear::One(b))
                    }
                    (Expression::Value(a), b) => {
                        self.parse_value(a);
                        let a = self.last;
                        self.parse_expression(b)?;
                        let b = self.last;
                        (a, b, Clear::One(a))
                    }
                    (a, b) => {
                        self.parse_expression(a)?;
                        let a = self.last;
                        self.parse_expression(b)?;
                        let b = self.last;
                        let next = self.next_register();
                        self.builder.add_binary_instruction(op, a, b, next);
                        return Ok(());
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
                    self.parse_expression(ex)?;
                    let r = self.last;
                    let next = self.next_register();
                    self.builder.add_unary_instruction(op, r, next);
                }
            },
            Expression::Identifier(id) => {
                if self.function_scopes.contains_key(id) {
                    self.call_function(None, id, vec![])?;
                } else {
                    let next = self.next_register();
                    self.builder.add_get_variable_instruction(id, next);
                }
            }
            Expression::If {
                condition,
                then,
                branch,
            } => {
                self.parse_expression(*condition)?;
                let cond = self.last;
                let (truthy, if_output) = self.parse_scope(then)?;
                match branch {
                    None => {
                        self.builder.add_if_instruction(cond, truthy, if_output);
                    }
                    Some(p) => {
                        let (falsy, else_output) = self.parse_scope(p)?;
                        let output = self.next_register();
                        self.builder.add_if_else_instruction(
                            cond,
                            (truthy, if_output),
                            (falsy, else_output),
                            output,
                        );
                    }
                }
            }
            Expression::Unless { condition, then } => {
                self.parse_expression(*condition)?;
                let cond = self.last;
                let (unless, output) = self.parse_scope(then)?;
                self.builder.add_unless_instruction(cond, unless, output);
            }
            Expression::List(list) => {
                self.parse_list(list)?;
            }
            Expression::Map(map) => {
                self.parse_map(map)?;
            }
            // todo use clear in function calls when appropriate
            Expression::FunctionCall(name, args) => {
                self.call_function(None, name, args)?;
            }
            // todo make a clear delineation between self.foo & Self.foo
            Expression::TypeFunctionCall(rigz_type, name, args) => {
                self.call_function(Some(rigz_type), name, args)?;
            }
            Expression::InstanceFunctionCall(exp, calls, args) => {
                let len = calls.len();
                assert!(len > 0, "Invalid Instance Function Call no arguments");
                let last = len - 1;
                let mut calls = calls.into_iter().enumerate();
                let (_, first) = calls.next().unwrap();
                let mut current = match self.function_scopes.contains_key(first) {
                    false => {
                        self.parse_expression(*exp)?;
                        let current = self.last;
                        let next = self.next_register();
                        self.builder.add_load_instruction(next, first.into());
                        let output = self.next_register();
                        self.builder
                            .add_instance_get_instruction(current, next, output);
                        output
                    }
                    true if last == 0 => {
                        self.call_extension_function(*exp, first, args)?;
                        return Ok(());
                    }
                    true => {
                        self.call_extension_function(*exp, first, vec![])?;
                        self.last
                    }
                };

                for (index, c) in calls {
                    let fcs = match self.function_scopes.get(c) {
                        None => {
                            let next = self.next_register();
                            self.builder.add_load_instruction(next, c.into());
                            let output = self.next_register();
                            self.builder
                                .add_instance_get_instruction(current, next, output);
                            current = output;
                            continue;
                        }
                        Some(fcs) => fcs.clone(),
                    };

                    if index == last {
                        self.call_inline_extension(c, fcs, &mut current, args)?;
                        break
                    } else {
                        self.call_inline_extension(c, fcs, &mut current, vec![])?;
                    }
                }
                self.last = current;
            }
            Expression::Scope(s) => {
                let (s, output) = self.parse_scope(s)?;
                let next = self.next_register();
                self.builder
                    .add_load_instruction(next, RegisterValue::ScopeId(s, output));
            }
            Expression::Cast(e, t) => {
                self.parse_expression(*e)?;
                let output = self.next_register();
                self.builder.add_cast_instruction(self.last, t, output);
            }
            Expression::Symbol(s) => {
                let next = self.next_register();
                // todo create a symbols cache in VM
                self.builder
                    .add_load_instruction(next, Value::String(s.to_string()).into());
            }
        }
        Ok(())
    }

    fn call_inline_extension(&mut self, name: &'vm str, function_call_signatures: FunctionCallSignatures<'vm>, current: &mut Register, args: Vec<Expression<'vm>>) -> Result<(), ValidationError> {
        let mut call_args = Vec::with_capacity(args.len());
        if function_call_signatures.len() == 1 {
            let mut fcs = function_call_signatures.into_iter();
            let (fcs, call) = fcs.next().unwrap();
            let (vm_module, mutable, this) = match &fcs.self_type {
                None => return Err(ValidationError::InvalidFunction(format!("Function is not an extension function {name}"))),
                Some((f, this)) => {
                    if this != current {
                        self.builder.add_load_instruction(*this, (*current).into());
                    }
                    (f.rigz_type == RigzType::VM, f.mutable, *this)
                }
            };
            self.setup_call_args(args, &mut call_args, fcs)?;
            self.process_extension_call(name, call_args, vm_module, mutable, this, call);
        } else {
            todo!("Multiple extensions match")
        }
        *current = self.last;
        Ok(())
    }

    fn parse_list(&mut self, list: Vec<Expression<'vm>>) -> Result<(), ValidationError> {
        let mut base = Vec::new();
        let mut values_only = true;
        let l = self.next_register();
        for (index, v) in list.into_iter().enumerate() {
            if values_only {
                match v {
                    Expression::Value(v) => {
                        base.push(v);
                    }
                    e => {
                        values_only = false;
                        self.builder
                            .add_load_instruction(l, Value::List(base).into());
                        base = Vec::new();
                        let index = Number::Int(index as i64);
                        let k_next = self.next_register();
                        self.parse_expression(e)?;
                        self.builder.add_load_instruction(k_next, index.into());
                        self.builder
                            .add_instance_set_instruction(l, k_next, self.last, l);
                    }
                }
            } else {
                let index = Number::Int(index as i64);
                let k_next = self.next_register();
                self.parse_expression(v)?;
                self.builder.add_load_instruction(k_next, index.into());
                self.builder
                    .add_instance_set_instruction(l, k_next, self.last, l);
            }
        }
        if values_only {
            self.builder
                .add_load_instruction(l, Value::List(base).into());
        } else {
            self.last = l;
        }
        Ok(())
    }

    fn parse_map(
        &mut self,
        map: Vec<(Expression<'vm>, Expression<'vm>)>,
    ) -> Result<(), ValidationError> {
        let mut base = IndexMap::new();
        let mut values_only = true;
        let m = self.next_register();
        for (k, v) in map {
            if values_only {
                match (k, v) {
                    (Expression::Value(k), Expression::Value(v)) => {
                        base.insert(k, v);
                    }
                    (Expression::Identifier(k), Expression::Value(v)) => {
                        base.insert(Value::String(k.to_string()), v);
                    }
                    (Expression::Identifier(k), e) => {
                        values_only = false;
                        self.builder
                            .add_load_instruction(m, Value::Map(base).into());
                        let k_next = self.next_register();
                        self.builder.add_load_instruction(k_next, k.into());
                        self.parse_expression(e)?;
                        self.builder
                            .add_instance_set_instruction(m, k_next, self.last, m);
                        base = IndexMap::new();
                    }
                    (k, v) => {
                        values_only = false;
                        self.builder
                            .add_load_instruction(m, Value::Map(base).into());
                        base = IndexMap::new();
                        self.parse_expression(k)?;
                        let k_next = self.last;
                        self.parse_expression(v)?;
                        self.builder
                            .add_instance_set_instruction(m, k_next, self.last, m);
                    }
                }
            } else {
                match (k, v) {
                    (Expression::Identifier(k), e) => {
                        let k_next = self.next_register();
                        self.builder.add_load_instruction(k_next, k.into());
                        self.parse_expression(e)?;
                        self.builder
                            .add_instance_set_instruction(m, k_next, self.last, m);
                    }
                    (k, v) => {
                        self.parse_expression(k)?;
                        let k_next = self.last;
                        self.parse_expression(v)?;
                        self.builder
                            .add_instance_set_instruction(m, k_next, self.last, m);
                    }
                }
            }
        }

        if values_only {
            self.builder
                .add_load_instruction(m, Value::Map(base).into());
        } else {
            self.last = m;
        }

        Ok(())
    }

    fn call_function(
        &mut self,
        rigz_type: Option<RigzType>,
        name: &'vm str,
        arguments: Vec<Expression<'vm>>,
    ) -> Result<(), ValidationError> {
        let function_call_signatures = self.get_function(name)?;
        let mut call_args = Vec::with_capacity(arguments.len());
        let mut fcs = None;
        let mut vm_module = false;
        let mut mutable = false;
        let mut this = 0;
        if function_call_signatures.len() == 1 {
            let (inner_fcs, call_site) = &function_call_signatures[0];
            if inner_fcs.arguments.len() == arguments.len() {
                match &inner_fcs.self_type {
                    None => {}
                    Some((ft, current_this)) => {
                        if ft.rigz_type == RigzType::VM {
                            vm_module = true
                        }
                        this = *current_this;
                        mutable = ft.mutable;
                    }
                }
                fcs = Some((inner_fcs.clone(), *call_site));
            } else {
                todo!("handle default and var args")
            }
        } else {
            for (fc, call_site) in function_call_signatures {
                match (&fc.self_type, &rigz_type) {
                    (None, None) => {
                        if fc.arguments.len() == arguments.len() {
                            fcs = Some((fc, call_site));
                            break;
                        } else {
                            todo!("check default and var args")
                        }
                    }
                    (Some((ft, current_this)), Some(s)) => {
                        if &ft.rigz_type == s {
                            if fc.arguments.len() == arguments.len() {
                                if s == &RigzType::VM {
                                    vm_module = true
                                }
                                this = *current_this;
                                mutable = ft.mutable;
                                fcs = Some((fc, call_site));
                                break;
                            } else {
                                todo!("check default and var args")
                            }
                        }
                    }
                    (None, Some(_)) | (Some(_), None) => {}
                }
            }
        }
        match fcs {
            None => {
                return Err(ValidationError::InvalidFunction(format!(
                    "No matching function found for {name} {rigz_type:?}, {arguments:?}"
                )))
            }
            Some((fcs, call)) => {
                self.setup_call_args(arguments, &mut call_args, fcs)?;

                match call {
                    CallSite::Scope(s, o) => {
                        self.builder.add_call_instruction(s, o);
                        let next = self.next_register();
                        self.builder.add_move_instruction(o, next);
                    }
                    CallSite::Module(m) => {
                        let output = self.next_register();
                        if vm_module {
                            self.builder.add_call_vm_extension_module_instruction(
                                m, name, call_args, output,
                            );
                        } else {
                            self.builder
                                .add_call_module_instruction(m, name, call_args, output);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn setup_call_args(&mut self, arguments: Vec<Expression<'vm>>, call_args: &mut Vec<Register>, fcs: FunctionCallSignature) -> Result<(), ValidationError> {
        for ((_, arg_reg), expression) in fcs.arguments.iter().zip(arguments) {
            self.parse_expression(expression)?;
            self.builder
                .add_load_instruction(*arg_reg, RegisterValue::Register(self.last));
            call_args.push(*arg_reg);
        }
        Ok(())
    }

    fn call_extension_function(
        &mut self,
        this_exp: Expression<'vm>,
        name: &'vm str,
        arguments: Vec<Expression<'vm>>,
    ) -> Result<(), ValidationError> {
        let function_call_signatures = self.get_function(name)?;
        let mut call_args = Vec::with_capacity(arguments.len());
        let mut fcs = None;
        let mut vm_module = false;
        let mut mutable = false;
        let mut this = 0;
        // todo add type inference for self
        let rigz_type = RigzType::Any;
        if function_call_signatures.len() == 1 {
            let (inner_fcs, call_site) = &function_call_signatures[0];
            if inner_fcs.arguments.len() == arguments.len() {
                match &inner_fcs.self_type {
                    None => {}
                    Some((ft, current_this)) => {
                        if ft.rigz_type == RigzType::VM {
                            vm_module = true
                        }
                        this = *current_this;
                        mutable = ft.mutable;
                    }
                }
                fcs = Some((inner_fcs.clone(), *call_site));
            } else {
                todo!("handle default and var args")
            }
        } else {
            for (fc, call_site) in function_call_signatures {
                match (&fc.self_type, &rigz_type) {
                    (None, _) => {}
                    (Some((ft, current_this)), s) => {
                        if &ft.rigz_type == s {
                            if fc.arguments.len() == arguments.len() {
                                if s == &RigzType::VM {
                                    vm_module = true
                                }
                                this = *current_this;
                                mutable = ft.mutable;
                                fcs = Some((fc, call_site));
                                break;
                            } else {
                                todo!("check default and var args")
                            }
                        }
                    }
                }
            }
        }
        match fcs {
            None => {
                return Err(ValidationError::InvalidFunction(format!(
                    "No matching function found for {name} {rigz_type:?}, {arguments:?}"
                )))
            }
            Some((fcs, call)) => {
                self.setup_call_args(arguments, &mut call_args, fcs)?;

                match &call {
                    CallSite::Scope(_, _) => {
                        self.parse_extension_expression(mutable, this, this_exp)?;
                    }
                    CallSite::Module(_) if vm_module => {

                    }
                    CallSite::Module(_) => {
                        self.parse_extension_expression(mutable, this, this_exp)?;
                    }
                }

                self.process_extension_call(name, call_args, vm_module, mutable, this, call);
            }
        }
        Ok(())
    }

    fn process_extension_call(&mut self, name: &'vm str, call_args: Vec<Register>, vm_module: bool, mutable: bool, this: Register, call: CallSite<'vm>) {
        match call {
            CallSite::Scope(s, o) => {
                self.builder.add_call_self_instruction(s, o, this, mutable);
                if mutable {
                    self.last = this;
                } else {
                    let next = self.next_register();
                    self.builder.add_move_instruction(o, next);
                }
            }
            CallSite::Module(m) => {
                let output = self.next_register();
                if vm_module {
                    self.builder.add_call_vm_extension_module_instruction(
                        m, name, call_args, output,
                    );
                } else {
                    if mutable {
                        self.builder.add_call_mutable_extension_module_instruction(
                            m, name, this, call_args, output,
                        );
                    } else {
                        self.builder.add_call_extension_module_instruction(
                            m, name, this, call_args, output,
                        );
                    }
                }
            }
        }
    }

    fn parse_extension_expression(
        &mut self,
        mutable: bool,
        this: Register,
        expression: Expression<'vm>,
    ) -> Result<(), ValidationError> {
        match expression {
            Expression::Identifier(id) => {
                if mutable {
                    self.builder.add_get_mutable_variable_instruction(id, this);
                } else {
                    let next = self.next_register();
                    self.builder.add_get_variable_instruction(id, next);
                }
            }
            _ => {
                self.parse_expression(expression)?;
            }
        }
        if !mutable && this != self.last {
            self.builder.add_load_instruction(this, self.last.into());
        }
        Ok(())
    }

    fn parse_import(&mut self, import: Exposed<'vm>) -> Result<(), ValidationError> {
        let name = match import {
            Exposed::TypeValue(tv) => tv,
            Exposed::Identifier(i) => {
                return Err(ValidationError::InvalidImport(format!(
                    "Identifier imports are not supported yet {i}"
                )))
            }
        };
        match self.modules.get_mut(name) {
            None => {
                // todo support non module imports
                return Err(ValidationError::ModuleError(format!(
                    "Module {name} does not exist"
                )));
            }
            Some(m) if !m.imported => {
                let old = std::mem::replace(m, ModuleTraitDefinition::imported(name));
                self.parse_trait_definition_for_module(name, old.definition)?;
            }
            Some(_) => {}
        };
        Ok(())
    }

    fn get_function(&self, name: &'vm str) -> Result<FunctionCallSignatures<'vm>, ValidationError> {
        match self.function_scopes.get(name) {
            None => Err(ValidationError::InvalidFunction(format!(
                "Function {name} does not exist"
            ))),
            Some(t) => Ok(t.clone()),
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

    // dont use this for function scopes!
    fn parse_scope(&mut self, scope: Scope<'vm>) -> Result<(usize, Register), ValidationError> {
        let current = self.builder.sp;
        self.builder.enter_scope();
        let res = self.builder.sp;
        for e in scope.elements {
            self.parse_element(e)?;
        }
        let next = self.last;
        self.builder.exit_scope(current, next);
        Ok((res, next))
    }

    pub(crate) fn build(mut self) -> VM<'vm> {
        self.builder.add_halt_instruction(self.last);
        self.builder.build()
    }
}
