mod program;

use crate::modules::{DateModule, NumberModule, StringModule, UUIDModule};
use crate::RuntimeError;
use crate::{FileModule, JSONModule, LogModule, RigzBuilder, StdModule, VMModule};
use log::Level;
pub use program::Program;
use rigz_ast::*;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum CallSite<'vm> {
    Scope(usize, Register, bool),
    Module(&'vm str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallSignature<'vm> {
    pub name: &'vm str,
    pub arguments: Vec<FunctionArgument<'vm>>,
    pub return_type: (FunctionType, Register),
    pub self_type: Option<(FunctionType, Register)>,
    pub arg_type: ArgType,
    pub var_args_start: Option<usize>,
}

impl<'vm> FunctionCallSignature<'vm> {
    pub(crate) fn convert(
        &self,
        args: RigzArguments<'vm>,
    ) -> Result<Vec<Expression<'vm>>, ValidationError> {
        match args {
            RigzArguments::Positional(a) => Ok(a),
            RigzArguments::Mixed(a, n) => {
                let mut args = a;
                let (_, rem) = self.arguments.split_at(args.len());
                args.extend(match_args(rem, n)?);
                Ok(args)
            }
            RigzArguments::Named(n) => {
                let (_, rem) = self.arguments.split_at(0);
                match_args(rem, n)
            }
        }
    }

    pub(crate) fn convert_ref<'a>(&self, args: &'a RigzArguments<'vm>) -> Vec<&'a Expression<'vm>> {
        match args {
            RigzArguments::Positional(a) => {
                let mut v = Vec::with_capacity(a.len());
                for e in a {
                    v.push(e)
                }
                v
            }
            RigzArguments::Mixed(a, n) => {
                let mut args = Vec::with_capacity(a.len());
                for e in a {
                    args.push(e)
                }
                let (_, rem) = self.arguments.split_at(args.len());
                args.extend(match_args_ref(rem, n));
                args
            }
            RigzArguments::Named(n) => {
                let (_, rem) = self.arguments.split_at(0);
                match_args_ref(rem, n)
            }
        }
    }
}

fn match_args<'vm>(
    rem: &[FunctionArgument<'vm>],
    named: Vec<(&str, Expression<'vm>)>,
) -> Result<Vec<Expression<'vm>>, ValidationError> {
    let mut res = Vec::with_capacity(rem.len());
    for (name, e) in named {
        for arg in rem {
            if arg.name == name {
                res.push(e.clone());
            }
        }
    }
    if res.len() != rem.len() {
        Err(ValidationError::InvalidFunction(format!(
            "Invalid # of args, expected {} got {}",
            rem.len(),
            res.len()
        )))
    } else {
        Ok(res)
    }
}

fn match_args_ref<'a, 'vm>(
    rem: &[FunctionArgument<'vm>],
    named: &'a Vec<(&str, Expression<'vm>)>,
) -> Vec<&'a Expression<'vm>> {
    let mut res = Vec::with_capacity(rem.len());
    for (name, e) in named {
        for arg in rem {
            if &arg.name == name {
                res.push(e);
            }
        }
    }
    res
}

#[derive(Clone)]
pub(crate) enum CallSignature<'vm> {
    Function(FunctionCallSignature<'vm>, CallSite<'vm>),
    Lambda(Vec<RigzType>, RigzType),
}

impl<'vm> CallSignature<'vm> {
    fn rigz_type(&self) -> RigzType {
        match self {
            CallSignature::Function(fc, _) => fc.return_type.0.rigz_type.clone(),
            CallSignature::Lambda(_, rt) => rt.clone(),
        }
    }
}

type FunctionCallSignatures<'vm> = Vec<CallSignature<'vm>>;

pub(crate) enum ModuleDefinition {
    Imported,
    Module(ModuleTraitDefinition<'static>),
}

pub(crate) struct ProgramParser<'vm, T: RigzBuilder<'vm>> {
    pub(crate) builder: T,
    pub(crate) current: Register,
    pub(crate) last: Register,
    pub(crate) modules: IndexMap<&'vm str, ModuleDefinition>,
    // todo nested functions are global, they should be removed if invalid
    pub(crate) function_scopes: IndexMap<&'vm str, FunctionCallSignatures<'vm>>,
    pub(crate) constants: IndexMap<Value, usize>,
    pub(crate) identifiers: HashMap<&'vm str, RigzType>,
    pub(crate) types: HashMap<&'vm str, RigzType>,
}

impl<'vm, T: RigzBuilder<'vm>> Default for ProgramParser<'vm, T> {
    fn default() -> Self {
        let mut builder = T::default();
        let none = builder.add_constant(Value::None);
        ProgramParser {
            builder,
            current: 0,
            last: 0,
            modules: Default::default(),
            function_scopes: Default::default(),
            constants: IndexMap::from([(Value::None, none)]),
            identifiers: Default::default(),
            types: Default::default(),
        }
    }
}

impl<'vm> ProgramParser<'vm, VMBuilder<'vm>> {
    pub(crate) fn create(self) -> ProgramParser<'vm, VM<'vm>> {
        let ProgramParser {
            builder,
            current,
            last,
            modules,
            function_scopes,
            constants,
            identifiers,
            types,
        } = self;
        ProgramParser {
            builder: builder.build(),
            current,
            last,
            modules,
            function_scopes,
            constants,
            identifiers,
            types,
        }
    }
}

impl<'vm> ProgramParser<'vm, VM<'vm>> {
    pub(crate) fn repl(&mut self, next_input: String) -> Result<&mut Self, RuntimeError> {
        let first = &mut self.builder.scopes[0];
        let last = first.instructions.len();
        if last > 0 {
            match first.instructions.remove(last - 1) {
                Instruction::Halt(r) => {
                    self.builder.current.borrow_mut().pc -= 1;
                    self.last = r;
                }
                i => {
                    first.instructions.push(i);
                }
            }
        }

        let p = parse(next_input.leak()).map_err(|e| e.into())?.into();
        self.parse_program(p).map_err(|e| e.into())?;
        Ok(self)
    }
}

struct BestMatch<'vm> {
    fcs: CallSignature<'vm>,
    mutable: bool,
    vm_module: bool,
    this: Option<usize>,
}

impl<'vm, T: RigzBuilder<'vm>> ProgramParser<'vm, T> {
    pub(crate) fn new() -> Self {
        let mut p = ProgramParser::default();
        p.add_default_modules();
        p
    }

    fn add_default_modules(&mut self) {
        self.register_module(VMModule);
        self.register_module(NumberModule);
        self.register_module(StringModule);
        self.register_module(StdModule);
        self.register_module(LogModule);
        self.register_module(JSONModule);
        self.register_module(FileModule);
        self.register_module(DateModule);
        self.register_module(UUIDModule);
    }

    pub(crate) fn register_module(&mut self, module: impl ParsedModule<'vm> + 'static) {
        let name = module.name();
        let def = module.module_definition();
        self.modules.insert(name, ModuleDefinition::Module(def));
        self.builder.register_module(module);
    }

    fn parse_module_trait_definition(
        &mut self,
        module: ModuleTraitDefinition<'static>,
    ) -> Result<(), ValidationError> {
        self.parse_trait_definition_for_module(module.definition.name, module.definition)
    }

    pub(crate) fn parse_program(&mut self, program: Program<'vm>) -> Result<(), ValidationError> {
        for element in program.elements {
            self.parse_element(element)?;
        }
        self.builder.add_halt_instruction(self.last);
        Ok(())
    }

    pub(crate) fn parse_element(&mut self, element: Element<'vm>) -> Result<(), ValidationError> {
        match element {
            Element::Statement(s) => self.parse_statement(s),
            Element::Expression(e) => self.parse_expression(e),
        }
    }

    fn parse_lambda(
        &mut self,
        name: &'vm str,
        arguments: Vec<FunctionArgument<'vm>>,
        var_args_start: Option<usize>,
        body: Box<Expression<'vm>>,
    ) -> Result<(), ValidationError> {
        let old: Vec<_> = arguments
            .iter()
            .map(|a| {
                (
                    a.name,
                    self.identifiers
                        .insert(a.name, a.function_type.rigz_type.clone()),
                )
            })
            .collect();
        let rigz_type = self.rigz_type(&body)?;
        let body = match *body {
            Expression::Scope(s) => s,
            ex => Scope {
                elements: vec![Element::Expression(ex)],
            },
        };
        let fd = FunctionDefinition {
            name,
            body,
            type_definition: FunctionSignature {
                arguments,
                return_type: FunctionType {
                    rigz_type,
                    mutable: false,
                },
                self_type: None,
                arg_type: ArgType::Positional,
                var_args_start,
            },
            lifecycle: None,
        };
        self.parse_function_definition(fd)?;
        old.into_iter().for_each(|(name, rt)| match rt {
            None => {
                self.identifiers.remove(name);
            }
            Some(s) => {
                self.identifiers.insert(name, s);
            }
        });
        Ok(())
    }

    fn parse_assignment(
        &mut self,
        lhs: Assign<'vm>,
        expression: Expression<'vm>,
    ) -> Result<(), ValidationError> {
        match lhs {
            Assign::Identifier(name, mutable) => match expression {
                Expression::Lambda {
                    arguments,
                    var_args_start,
                    body,
                } => self.parse_lambda(name, arguments, var_args_start, body)?,
                exp => {
                    let ext = self.rigz_type(&exp)?;
                    self.identifiers.insert(name, ext);
                    self.parse_expression(exp)?;
                    if mutable {
                        self.builder.add_load_mut_instruction(name, self.last);
                    } else {
                        self.builder.add_load_let_instruction(name, self.last);
                    }
                }
            },
            Assign::TypedIdentifier(name, mutable, rigz_type) => {
                match expression {
                    Expression::Lambda {
                        arguments,
                        var_args_start,
                        body,
                    } => {
                        // todo ensure lambda matches typed identifier
                        self.parse_lambda(name, arguments, var_args_start, body)?
                    }
                    exp => {
                        let ext = self.rigz_type(&exp)?;
                        if ext != rigz_type {
                            return Err(ValidationError::InvalidType(format!(
                                "{ext} cannot be assigned to {rigz_type}"
                            )));
                        }
                        self.identifiers.insert(name, ext);
                        self.parse_expression(exp)?;
                        if mutable {
                            self.builder.add_load_mut_instruction(name, self.last);
                        } else {
                            self.builder.add_load_let_instruction(name, self.last);
                        }
                    }
                }
            }
            Assign::This => {
                let this = self.mutable_this();
                match expression {
                    Expression::Lambda {
                        arguments,
                        var_args_start,
                        body,
                    } => self.parse_lambda("self", arguments, var_args_start, body)?,
                    exp => {
                        let ext = self.rigz_type(&exp)?;
                        self.identifiers.insert("self", ext);
                        self.parse_expression(exp)?;
                        let rhs = self.last;
                        self.builder
                            .add_load_instruction(this, RegisterValue::Register(rhs));
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn parse_statement(
        &mut self,
        statement: Statement<'vm>,
    ) -> Result<(), ValidationError> {
        match statement {
            Statement::Assignment { lhs, expression } => self.parse_assignment(lhs, expression)?,
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
            Statement::BinaryAssignment {
                lhs: Assign::TypedIdentifier(name, _, _),
                op,
                expression,
            } => {
                let this = self.next_register();
                self.builder
                    .add_get_mutable_variable_instruction(name, this);
                // todo validate expression is rigz_type
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
            Statement::TypeDefinition(name, def) => {
                self.types.insert(name, def);
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
            lifecycle,
        } = function_definition;
        let identifiers = self.identifiers.clone();
        let type_definition = self.parse_type_signature(name, type_definition)?;
        let current_scope = self.builder.current_scope();
        let args = type_definition
            .arguments
            .iter()
            .map(|a| (a.name, a.function_type.mutable))
            .collect();
        let memoized = match lifecycle {
            None => {
                self.builder.enter_scope(name, args);
                false
            }
            Some(l) => {
                let memoized = match &l {
                    Lifecycle::Memo(_) => true,
                    Lifecycle::Composite(all) => {
                        all.iter().any(|l| matches!(l, Lifecycle::Memo(_)))
                    }
                    _ => false,
                };
                self.builder.enter_lifecycle_scope(name, l, args);
                memoized
            }
        };
        for arg in &type_definition.arguments {
            let rt = &arg.function_type.rigz_type;
            match rt {
                RigzType::Function(args, ret) => {
                    let cs = CallSignature::Lambda(args.clone(), *ret.clone());
                    match self.function_scopes.entry(arg.name) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().push(cs);
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(vec![cs]);
                        }
                    }
                }
                _ => {
                    self.identifiers.insert(arg.name, rt.clone());
                }
            }
        }
        // todo store arguments variable
        let f_def = self.builder.current_scope();
        let (_, output) = type_definition.return_type;
        let self_type = type_definition.self_type.clone();
        match self.function_scopes.entry(name) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(CallSignature::Function(
                    type_definition,
                    CallSite::Scope(f_def, output, memoized),
                ));
            }
            Entry::Vacant(e) => {
                e.insert(vec![CallSignature::Function(
                    type_definition,
                    CallSite::Scope(f_def, output, memoized),
                )]);
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
        self.identifiers = identifiers;
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
                    let type_definition = self.parse_type_signature(name, type_definition)?;
                    match self.function_scopes.entry(name) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().push(CallSignature::Function(
                                type_definition,
                                CallSite::Module(module_name),
                            ));
                        }
                        Entry::Vacant(e) => {
                            e.insert(vec![CallSignature::Function(
                                type_definition,
                                CallSite::Module(module_name),
                            )]);
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
        name: &'vm str,
        function_signature: FunctionSignature<'vm>,
    ) -> Result<FunctionCallSignature<'vm>, ValidationError> {
        let FunctionSignature {
            arguments,
            return_type,
            self_type,
            arg_type,
            var_args_start,
        } = function_signature;
        let self_type = self_type.map(|f| (f, self.next_register()));
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
            name,
            arguments,
            return_type,
            self_type,
            arg_type,
            var_args_start,
        })
    }

    pub(crate) fn parse_trait_definition(
        &mut self,
        trait_definition: TraitDefinition<'vm>,
    ) -> Result<(), ValidationError> {
        for func in trait_definition.functions {
            match func {
                FunctionDeclaration::Declaration {
                    ..
                } => {
                    return Err(ValidationError::NotImplemented("Support Function Declarations in Trait definition, call site is determined by impl".to_string()))
                }
                FunctionDeclaration::Definition(fd) => self.parse_function_definition(fd)?,
            }
        }
        Ok(())
    }

    fn check_module_exists(&mut self, function: &str) -> Result<(), ValidationError> {
        let modules: Vec<_> = self
            .modules
            .iter()
            .filter_map(|(_, m)| match m {
                ModuleDefinition::Imported => None,
                ModuleDefinition::Module(m) => {
                    if m.auto_import
                        && m.definition.functions.iter().any(|f| match f {
                            FunctionDeclaration::Declaration { name, .. } => *name == function,
                            FunctionDeclaration::Definition(f) => f.name == function,
                        })
                    {
                        Some(m.definition.name)
                    } else {
                        None
                    }
                }
            })
            .collect();
        for m in modules {
            // todo support parsing only required functions
            self.parse_import(ImportValue::TypeValue(m))?;
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
            Expression::Tuple(v) => {
                self.parse_tuple(v)?;
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
                    self.call_function(None, id, vec![].into())?;
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
                let (truthy, if_output) = self.parse_scope(then, "if")?;
                match branch {
                    None => {
                        self.builder.add_if_instruction(cond, truthy, if_output);
                    }
                    Some(p) => {
                        let (falsy, else_output) = self.parse_scope(p, "else")?;
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
                let (unless, output) = self.parse_scope(then, "unless")?;
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
            Expression::Lambda { .. } => {
                return Err(ValidationError::MissingExpression(
                    "Dangling lambda cannot be used".to_string(),
                ))
            }
            Expression::ForList {
                var,
                expression: exp,
                body,
            } => {
                let current = self.builder.current_scope();
                // todo extract type from expression
                let old = self.identifiers.insert(var, RigzType::Any);
                let inner_scope = self.builder.enter_scope("for-list", vec![(var, false)]);
                self.parse_expression(*body)?;
                let output = self.last;
                self.builder.exit_scope(current, output);
                match old {
                    None => {
                        self.identifiers.remove(var);
                    }
                    Some(t) => {
                        self.identifiers.insert(var, t);
                    }
                }
                self.parse_expression(*exp)?;
                let this = self.last;
                self.last = output;
                self.builder
                    .add_for_list_instruction(this, inner_scope, output);
            }
            Expression::ForMap {
                k_var,
                v_var,
                expression,
                key,
                value,
            } => {
                if k_var == v_var {
                    return Err(ValidationError::MissingExpression(format!(
                        "Cannot use same identifier for key & value, {k_var}"
                    )));
                }

                let current = self.builder.current_scope();
                let k_old = self.identifiers.insert(k_var, RigzType::Any);
                let v_old = self.identifiers.insert(v_var, RigzType::Any);
                let inner_scope = self
                    .builder
                    .enter_scope("for-map", vec![(k_var, false), (v_var, false)]);
                match value {
                    None => {
                        self.parse_expression(*key)?;
                    }
                    Some(value) => {
                        self.parse_tuple(vec![*key, *value])?;
                    }
                }
                let output = self.last;
                self.builder.exit_scope(current, output);
                match k_old {
                    None => {
                        self.identifiers.remove(k_var);
                    }
                    Some(t) => {
                        self.identifiers.insert(k_var, t);
                    }
                }
                match v_old {
                    None => {
                        self.identifiers.remove(k_var);
                    }
                    Some(t) => {
                        self.identifiers.insert(k_var, t);
                    }
                }
                let key = self.next_register();
                let value = self.next_register();
                self.parse_expression(*expression)?;
                let this = self.last;
                self.last = output;
                self.builder
                    .add_for_map_instruction(this, inner_scope, key, value, output);
            }
            Expression::InstanceFunctionCall(exp, calls, args) => {
                let len = calls.len();
                assert!(len > 0, "Invalid Instance Function Call no calls");
                let last = len - 1;
                let mut calls = calls.into_iter().enumerate();
                let (_, first) = calls.next().unwrap();
                self.check_module_exists(first)?;
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
                        self.call_extension_function(*exp, first, vec![].into())?;
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
                        break;
                    } else {
                        self.call_inline_extension(c, fcs, &mut current, vec![].into())?;
                    }
                }
                self.last = current;
            }
            Expression::Scope(s) => {
                let (s, output) = self.parse_scope(s, "do")?;
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
                let index = self.find_or_create_constant(s.into());
                self.builder
                    .add_load_instruction(next, RegisterValue::Constant(index));
            }
            Expression::Return(ret) => {
                let reg = match ret {
                    None => {
                        let none = self.find_or_create_constant(Value::None);
                        let next = self.next_register();
                        self.builder
                            .add_load_instruction(next, RegisterValue::Constant(none));
                        next
                    }
                    Some(e) => {
                        self.parse_expression(*e)?;
                        self.last
                    }
                };
                self.builder.add_ret_instruction(reg);
            }
        }
        Ok(())
    }

    fn find_or_create_constant(&mut self, value: Value) -> usize {
        match self.constants.entry(value) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let index = self.builder.add_constant(e.key().clone());
                e.insert(index);
                index
            }
        }
    }

    fn call_inline_extension(
        &mut self,
        name: &'vm str,
        function_call_signatures: FunctionCallSignatures<'vm>,
        current: &mut Register,
        args: RigzArguments<'vm>,
    ) -> Result<(), ValidationError> {
        if function_call_signatures.len() == 1 {
            let mut fcs = function_call_signatures.into_iter();
            match fcs.next().unwrap() {
                CallSignature::Function(fcs, call) => {
                    let (vm_module, mutable, this) = match &fcs.self_type {
                        None => {
                            return Err(ValidationError::InvalidFunction(format!(
                                "Function is not an extension function {name}"
                            )))
                        }
                        Some((f, this)) => {
                            if this != current {
                                self.builder.add_load_instruction(*this, (*current).into());
                            }
                            let is_vm =
                                if let RigzType::Custom(CustomType { name, .. }) = &f.rigz_type {
                                    name.as_str() == "VM"
                                } else {
                                    false
                                };
                            (is_vm, f.mutable, *this)
                        }
                    };
                    let call_args = self.setup_call_args(args, fcs)?;
                    self.process_extension_call(name, call_args, vm_module, mutable, this, call);
                }
                CallSignature::Lambda(args, ret) => {}
            };
        } else {
            return Err(ValidationError::NotImplemented(format!(
                "Multiple extension functions match for {name}"
            )));
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
        }
        self.last = l;
        Ok(())
    }

    fn parse_tuple(&mut self, list: Vec<Expression<'vm>>) -> Result<(), ValidationError> {
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
                            .add_load_instruction(l, Value::Tuple(base).into());
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
                .add_load_instruction(l, Value::Tuple(base).into());
        }
        self.last = l;
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
        }
        self.last = m;

        Ok(())
    }

    fn str_to_log_level(raw: &str) -> Result<Level, ValidationError> {
        let level = match raw.to_lowercase().as_str() {
            "info" => Level::Info,
            "warn" => Level::Info,
            "debug" => Level::Debug,
            "trace" => Level::Trace,
            "error" => Level::Error,
            s => {
                return Err(ValidationError::InvalidFunction(format!(
                    "Invalid log level {s}"
                )))
            }
        };
        Ok(level)
    }

    fn load_none(&mut self) {
        let next = self.next_register();
        self.builder
            .add_load_instruction(next, RegisterValue::Constant(0));
    }

    fn call_function(
        &mut self,
        rigz_type: Option<RigzType>,
        name: &'vm str,
        arguments: RigzArguments<'vm>,
    ) -> Result<(), ValidationError> {
        self.check_module_exists(name)?;
        // todo this should check if another function named puts exists first
        if name == "puts" {
            if let RigzArguments::Positional(arguments) = arguments {
                let mut args = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    self.parse_expression(arg)?;
                    args.push(self.last);
                }
                self.builder.add_puts_instruction(args);
                self.load_none();
                return Ok(());
            }
        }

        // todo this should check if another function named log exists first
        if name == "log" {
            if let RigzArguments::Positional(arguments) = &arguments {
                if arguments.len() >= 2 {
                    let mut args = Vec::with_capacity(arguments.len() - 2);
                    let mut arguments = arguments.iter();
                    let level = match arguments.next().unwrap() {
                        Expression::Value(Value::String(s)) => Self::str_to_log_level(s.as_str())?,
                        Expression::Symbol(s) => Self::str_to_log_level(s)?,
                        // todo support identifiers here
                        e => {
                            return Err(ValidationError::InvalidFunction(format!(
                                "Unable to create log level for {e:?}, must be string or symbol"
                            )))
                        }
                    };

                    let template = match arguments.next().unwrap() {
                        Expression::Value(Value::String(s)) => s.clone(),
                        _ => "{}".to_string(),
                    };

                    for arg in arguments {
                        self.parse_expression(arg.clone())?;
                        args.push(self.last);
                    }
                    self.builder
                        .add_log_instruction(level, template.leak(), args);
                    self.load_none();
                    return Ok(());
                }
            }
        }

        let BestMatch {
            fcs,
            mutable: _,
            vm_module,
            this: _,
        } = self.best_matched_function(name, rigz_type, &arguments)?;

        match fcs {
            CallSignature::Function(fcs, call) => {
                let call_args = self.setup_call_args(arguments, fcs)?;

                match call {
                    CallSite::Scope(s, o, memo) => {
                        if memo {
                            self.builder.add_call_memo_instruction(s, call_args, o);
                        } else {
                            self.builder.add_call_instruction(s, call_args, o);
                        }
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
            CallSignature::Lambda(args, ret) => {}
        };
        Ok(())
    }

    fn best_matched_function(
        &self,
        name: &'vm str,
        rigz_type: Option<RigzType>,
        arguments: &RigzArguments<'vm>,
    ) -> Result<BestMatch<'vm>, ValidationError> {
        let mut fcs = None;
        let mut vm_module = false;
        let mut this = None;
        let mut mutable = false;

        let function_call_signatures = self.get_function(name)?;
        if function_call_signatures.len() == 1 {
            match &function_call_signatures[0] {
                CallSignature::Function(inner_fcs, call_site) => {
                    let arguments = inner_fcs.convert_ref(arguments);
                    if arguments.len() <= inner_fcs.arguments.len() {
                        match &inner_fcs.self_type {
                            None => {}
                            Some((ft, current_this)) => {
                                vm_module = ft.rigz_type.is_vm();
                                this = Some(*current_this);
                                mutable = ft.mutable;
                            }
                        }
                        fcs = Some(CallSignature::Function(inner_fcs.clone(), *call_site));
                    } else if inner_fcs.var_args_start.is_none() {
                        return Err(ValidationError::InvalidFunction(format!(
                            "Expected function with var_args {name}"
                        )));
                    }
                }
                lambda => fcs = Some(lambda.clone()),
            }
        } else {
            for cs in function_call_signatures {
                match cs {
                    CallSignature::Function(fc, call_site) => {
                        let arguments = fc.convert_ref(arguments);
                        let arg_len = arguments.len();
                        let fc_arg_len = fc.arguments.len();
                        match (&fc.self_type, &rigz_type) {
                            (None, None) => {
                                if arg_len <= fc_arg_len {
                                    fcs = Some(CallSignature::Function(fc, call_site));
                                    break;
                                } else {
                                    match fc.var_args_start {
                                        None => {}
                                        Some(i) => {
                                            if (fc_arg_len - 1) % (arg_len - i) == 0 {
                                                fcs = Some(CallSignature::Function(fc, call_site));
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            (Some((ft, current_this)), Some(s)) => {
                                if &ft.rigz_type == s {
                                    if arg_len <= fc_arg_len {
                                        vm_module = ft.rigz_type.is_vm();
                                        this = Some(*current_this);
                                        mutable = ft.mutable;
                                        fcs = Some(CallSignature::Function(fc, call_site));
                                        break;
                                    } else {
                                        match fc.var_args_start {
                                            None => {}
                                            Some(i) => {
                                                if (fc_arg_len - 1) % (arg_len - i) == 0 {
                                                    mutable = ft.mutable;
                                                    this = Some(*current_this);
                                                    fcs = Some(CallSignature::Function(
                                                        fc, call_site,
                                                    ));
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            (None, Some(_)) | (Some(_), None) => {}
                        }
                    }
                    CallSignature::Lambda(args, ret) => {
                        // todo ensure best match
                        fcs = Some(CallSignature::Lambda(args, ret))
                    }
                }
            }
        }
        match fcs {
            None => match rigz_type {
                None => Err(ValidationError::InvalidFunction(format!(
                    "No matching function found for {name}"
                ))),
                Some(r) => Err(ValidationError::InvalidFunction(format!(
                    "No matching function found for {r}.{name}"
                ))),
            },
            Some(fcs) => Ok(BestMatch {
                fcs,
                mutable,
                vm_module,
                this,
            }),
        }
    }

    fn setup_call_args(
        &mut self,
        arguments: RigzArguments<'vm>,
        fcs: FunctionCallSignature<'vm>,
    ) -> Result<Vec<Register>, ValidationError> {
        let arguments = fcs.convert(arguments)?;
        let arg_len = arguments.len();
        let mut call_args = Vec::with_capacity(arg_len);
        let arguments = if arg_len < fcs.arguments.len() {
            let mut arguments = arguments;
            let (_, rem) = fcs.arguments.split_at(arg_len);
            for arg in rem {
                if arg.default.is_none() {
                    return Err(ValidationError::MissingExpression(format!(
                        "Invalid args for {} expected default value for {arg:?}",
                        fcs.name
                    )));
                }
                // todo should these be constants?
                arguments.push(Expression::Value(arg.default.clone().unwrap()))
            }
            arguments
        } else {
            match fcs.var_args_start {
                None => arguments,
                Some(i) => {
                    let (base, vars) = arguments.split_at(i);
                    let mut args = base.to_vec();
                    let var_arg_count = i - args.len() + 1;
                    let mut a = Vec::with_capacity(var_arg_count);
                    a.resize(var_arg_count, Vec::new());
                    let mut last_var_arg = 0;
                    for (index, ex) in vars.iter().enumerate() {
                        last_var_arg = index % var_arg_count;
                        a[last_var_arg].push(ex.clone());
                    }

                    if last_var_arg % var_arg_count != 0 {
                        let (_, rem) = fcs.arguments.split_at(i);
                        for (index, arg) in rem.iter().enumerate() {
                            if arg.default.is_none() {
                                return Err(ValidationError::MissingExpression(format!(
                                    "Invalid var_args for {} expected default value for {arg:?}",
                                    fcs.name
                                )));
                            }
                            // todo should these be constants?
                            a[index + last_var_arg]
                                .push(Expression::Value(arg.default.clone().unwrap()))
                        }
                    }
                    args.extend(a.into_iter().map(Expression::List));
                    args
                }
            }
        };
        if arguments.len() != fcs.arguments.len() {
            return Err(ValidationError::InvalidFunction(format!(
                "Missing arguments for {}",
                fcs.name
            )));
        }
        for (arg, expression) in fcs.arguments.iter().zip(arguments) {
            match expression {
                Expression::Lambda {
                    arguments,
                    var_args_start,
                    body,
                } => {
                    self.parse_lambda(arg.name, arguments, var_args_start, body)?;
                }
                _ => {
                    self.parse_expression(expression)?;
                }
            }
            call_args.push(self.last);
        }
        Ok(call_args)
    }

    fn call_extension_function(
        &mut self,
        this_exp: Expression<'vm>,
        name: &'vm str,
        arguments: RigzArguments<'vm>,
    ) -> Result<(), ValidationError> {
        let rigz_type = self.rigz_type(&this_exp)?;
        let BestMatch {
            fcs,
            mutable,
            vm_module,
            this,
        } = self.best_matched_function(name, Some(rigz_type), &arguments)?;
        match fcs {
            CallSignature::Function(fcs, call) => {
                let call_args = self.setup_call_args(arguments, fcs)?;

                let this = match this {
                    None => {
                        return Err(ValidationError::InvalidFunction(format!(
                            "Invalid function matched for {name}, required extension function"
                        )))
                    }
                    Some(t) => t,
                };

                match &call {
                    CallSite::Scope(_, _, _) => {
                        self.parse_extension_expression(mutable, this, this_exp)?;
                    }
                    CallSite::Module(_) => {
                        self.parse_extension_expression(mutable, this, this_exp)?;
                    }
                }

                self.process_extension_call(name, call_args, vm_module, mutable, this, call);
            }
            CallSignature::Lambda(args, ret) => {}
        };
        Ok(())
    }

    fn process_extension_call(
        &mut self,
        name: &'vm str,
        call_args: Vec<Register>,
        vm_module: bool,
        mutable: bool,
        this: Register,
        call: CallSite<'vm>,
    ) {
        match call {
            CallSite::Scope(s, o, memo) => {
                if memo {
                    self.builder
                        .add_call_self_memo_instruction(s, call_args, o, this, mutable);
                } else {
                    self.builder
                        .add_call_self_instruction(s, call_args, o, this, mutable);
                }
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
                    self.builder
                        .add_call_vm_extension_module_instruction(m, name, call_args, output);
                } else if mutable {
                    self.builder.add_call_mutable_extension_module_instruction(
                        m, name, this, call_args, output,
                    );
                } else {
                    self.builder
                        .add_call_extension_module_instruction(m, name, this, call_args, output);
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

    fn parse_import(&mut self, import: ImportValue<'vm>) -> Result<(), ValidationError> {
        let name = match import {
            ImportValue::TypeValue(tv) => tv,
            ImportValue::FilePath(f) => {
                return Err(ValidationError::NotImplemented(format!(
                    "File imports are not supported yet {f}"
                )))
            }
            ImportValue::UrlPath(url) => {
                return Err(ValidationError::NotImplemented(format!(
                    "Url imports are not supported yet {url}"
                )))
            }
        };

        match self.modules.entry(name) {
            Entry::Occupied(mut m) => {
                let def = m.get_mut();
                if let ModuleDefinition::Module(_) = def {
                    let ModuleDefinition::Module(def) =
                        std::mem::replace(def, ModuleDefinition::Imported)
                    else {
                        unreachable!()
                    };
                    self.parse_module_trait_definition(def)?;
                } else {
                    return Ok(());
                }
            }
            Entry::Vacant(_) => {
                // todo support non module imports
                return Err(ValidationError::ModuleError(format!(
                    "Module {name} does not exist"
                )));
            }
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
    fn parse_scope(
        &mut self,
        scope: Scope<'vm>,
        named: &'vm str,
    ) -> Result<(usize, Register), ValidationError> {
        let current_vars = self.identifiers.clone();
        let current = self.builder.current_scope();
        self.builder.enter_scope(named, vec![]);
        let res = self.builder.current_scope();
        for e in scope.elements {
            self.parse_element(e)?;
        }
        let next = self.last;
        self.builder.exit_scope(current, next);
        self.identifiers = current_vars;
        Ok((res, next))
    }
}
