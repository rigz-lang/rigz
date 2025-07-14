mod program;

use crate::RuntimeError;
use itertools::Itertools;
use log::{error, warn, Level};
pub use program::Program;
use rigz_ast::*;
use rigz_core::{
    EnumDeclaration, IndexMap, IndexMapEntry, Lifecycle, Number, ObjectValue, PrimitiveValue,
    RigzType,
};
use rigz_vm::{Instruction, LoadValue, MatchArm, RigzBuilder, VMBuilder, VM};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CallSite {
    Scope(usize, bool),
    Module(String),
    Object(usize),
    // todo only store used functions in VM
    // Parsed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallSignature {
    pub name: String,
    pub arguments: Vec<FunctionArgument>,
    pub return_type: FunctionType,
    pub self_type: Option<FunctionType>,
    pub arg_type: ArgType,
    pub var_args_start: Option<usize>,
}

impl FunctionCallSignature {
    pub(crate) fn convert(&self, args: RigzArguments) -> Result<Vec<Expression>, ValidationError> {
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

    pub(crate) fn convert_ref<'a>(&self, args: &'a RigzArguments) -> Vec<&'a Expression> {
        match args {
            RigzArguments::Positional(a) => a.iter().collect(),
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

fn match_args(
    rem: &[FunctionArgument],
    named: Vec<(String, Expression)>,
) -> Result<Vec<Expression>, ValidationError> {
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

fn match_args_ref<'a>(
    rem: &[FunctionArgument],
    named: &'a Vec<(String, Expression)>,
) -> Vec<&'a Expression> {
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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum CallSignature {
    Function(FunctionCallSignature, CallSite),
    // call signature is owning function
    Lambda(FunctionCallSignature, Vec<RigzType>, RigzType),
}

impl CallSignature {
    fn rigz_type(&self) -> RigzType {
        match self {
            CallSignature::Function(fc, _) => fc.return_type.rigz_type.clone(),
            CallSignature::Lambda(_, _, rt) => rt.clone(),
        }
    }
}

type FunctionCallSignatures = Vec<CallSignature>;

#[derive(Debug)]
pub(crate) enum ModuleDefinition {
    Imported,
    Module(ModuleTraitDefinition),
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Imports {
    root: usize,
}

#[derive(Debug)]
enum ObjectConstructor {
    Scope(Vec<FunctionArgument>, Option<usize>, usize),
    Custom(Vec<FunctionArgument>, Option<usize>),
}

#[derive(Debug)]
struct ObjectDeclaration {
    constructor: ObjectConstructor,
    rigz_type: Arc<RigzType>,
    fields: Vec<ObjectAttr>,
    dep: Option<usize>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum ImportPath {
    Url(String),
    File(PathBuf),
}

#[derive(Debug)]
pub(crate) struct ProgramParser<'vm, T: RigzBuilder> {
    pub(crate) builder: T,
    pub(crate) modules: IndexMap<&'vm str, ModuleDefinition>,
    // todo nested functions are global, they should be removed if invalid
    pub(crate) function_scopes: IndexMap<String, FunctionCallSignatures>,
    pub(crate) constants: IndexMap<ObjectValue, usize>,
    pub(crate) identifiers: HashMap<String, FunctionType>,
    pub(crate) types: HashMap<String, RigzType>,
    pub(crate) parser_options: ParserOptions,
    // todo imports should be fully resolved path
    imports: HashMap<ImportPath, Imports>,
    objects: HashMap<String, Rc<ObjectDeclaration>>,
    enums: HashMap<String, (usize, Arc<EnumDeclaration>)>,
    enum_lookups: HashMap<usize, Arc<EnumDeclaration>>,
    in_loop: bool
}

impl<T: RigzBuilder> Default for ProgramParser<'_, T> {
    fn default() -> Self {
        let mut builder = T::default();
        let none = builder.add_constant(ObjectValue::default());
        ProgramParser {
            builder,
            modules: Default::default(),
            function_scopes: Default::default(),
            constants: IndexMap::from([(ObjectValue::default(), none)]),
            identifiers: Default::default(),
            types: Default::default(),
            parser_options: Default::default(),
            imports: Default::default(),
            objects: Default::default(),
            enums: Default::default(),
            enum_lookups: Default::default(),
            in_loop: false,
        }
    }
}

impl<'vm> ProgramParser<'vm, VMBuilder> {
    pub(crate) fn create(self) -> ProgramParser<'vm, VM> {
        let ProgramParser {
            builder,
            modules,
            function_scopes,
            constants,
            identifiers,
            types,
            parser_options,
            imports,
            objects,
            enums,
            enum_lookups,
            in_loop
        } = self;
        ProgramParser {
            builder: builder.build(),
            modules,
            function_scopes,
            constants,
            identifiers,
            types,
            parser_options,
            imports,
            objects,
            enums,
            enum_lookups,
            in_loop
        }
    }
}

impl ProgramParser<'_, VM> {
    pub(crate) fn repl(&mut self, next_input: String) -> Result<&mut Self, RuntimeError> {
        let first = &mut self.builder.scopes[0];
        let last = first.instructions.len();
        if last > 0 {
            match first.instructions.remove(last - 1) {
                Instruction::Halt => {
                    self.builder.frames.current.borrow_mut().pc -= 1;
                }
                i => {
                    first.instructions.push(i);
                }
            }
        }

        let p = parse(next_input.as_str(), self.parser_options.clone())
            .map_err(|e| e.into())?
            .into();
        self.parse_program(p).map_err(|e| e.into())?;
        Ok(self)
    }
}

struct BestMatch {
    fcs: CallSignature,
    mutable: bool,
    vm_module: bool,
}

impl<T: RigzBuilder> ProgramParser<'_, T> {
    pub(crate) fn new() -> Self {
        let mut p = ProgramParser::default();
        p.add_default_modules()
            .expect("failed to register default modules");
        p
    }

    pub(crate) fn with_options(parser_options: ParserOptions) -> Self {
        let mut p = ProgramParser {
            parser_options,
            ..Default::default()
        };
        p.add_default_modules()
            .expect("failed to register default modules");
        p
    }

    pub(crate) fn register_module<M: ParsedModule + 'static>(
        &mut self,
        module: M,
    ) -> Result<(), ValidationError> {
        let name = M::name();
        let def = M::module_definition();
        for dep in M::parsed_dependencies() {
            let obj = dep.object_definition;
            let dep = self.builder.register_dependency(Arc::new(dep.dependency));
            self.parse_object_definition(obj, Some(dep))?;
        }
        self.modules.insert(name, ModuleDefinition::Module(def));
        self.builder.register_module(module);
        Ok(())
    }

    fn parse_module_trait_definition(
        &mut self,
        module: ModuleTraitDefinition,
    ) -> Result<(), ValidationError> {
        self.parse_trait_definition_for_module(module.definition)
    }

    pub(crate) fn parse_program(&mut self, program: Program) -> Result<(), ValidationError> {
        self.parse_scoped_program(program, None)
    }

    pub(crate) fn parse_scoped_program(
        &mut self,
        program: Program,
        current: Option<usize>,
    ) -> Result<(), ValidationError> {
        for element in program.elements {
            self.parse_element(element)?;
        }
        match current {
            None => {
                self.builder.add_halt_instruction();
            }
            Some(s) => {
                self.builder.exit_scope(s);
            }
        }
        Ok(())
    }

    pub(crate) fn parse_element(&mut self, element: Element) -> Result<(), ValidationError> {
        match element {
            Element::Statement(s) => self.parse_statement(s),
            Element::Expression(e) => self.parse_expression(e),
        }
    }

    fn parse_lambda(
        &mut self,
        name: &str,
        arguments: Vec<FunctionArgument>,
        var_args_start: Option<usize>,
        body: Box<Expression>,
    ) -> Result<(), ValidationError> {
        let old: Vec<_> = arguments
            .iter()
            .map(|a| {
                (
                    a.name.clone(),
                    self.identifiers
                        .insert(a.name.clone(), a.function_type.clone()),
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
            name: name.to_string(),
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
                self.identifiers.remove(&name);
            }
            Some(s) => {
                *self.identifiers.get_mut(&name).unwrap() = s;
            }
        });
        Ok(())
    }

    fn parse_assignment(
        &mut self,
        lhs: Assign,
        expression: Expression,
    ) -> Result<(), ValidationError> {
        match lhs {
            Assign::Identifier {
                name,
                mutable,
                shadow,
            } => match expression {
                Expression::Lambda {
                    arguments,
                    var_args_start,
                    body,
                } => self.parse_lambda(&name, arguments, var_args_start, body)?,
                exp => {
                    let ext = self.rigz_type(&exp)?;
                    self.parse_lazy_expression(exp, &name)?;
                    let var = name.to_string();
                    let mutable = match self.identifiers.entry(name) {
                        Entry::Occupied(mut t) => {
                            let v = t.get();
                            if v.rigz_type == ext {
                                v.mutable
                            } else {
                                t.insert(FunctionType {
                                    rigz_type: ext,
                                    mutable,
                                });
                                mutable
                            }
                        }
                        Entry::Vacant(v) => {
                            v.insert(FunctionType {
                                rigz_type: ext,
                                mutable,
                            });
                            mutable
                        }
                    };
                    if mutable {
                        self.builder.add_load_mut_instruction(var, shadow);
                    } else {
                        self.builder.add_load_let_instruction(var, shadow);
                    }
                }
            },
            Assign::TypedIdentifier {
                name,
                mutable,
                rigz_type,
                shadow,
            } => {
                match expression {
                    Expression::Lambda {
                        arguments,
                        var_args_start,
                        body,
                    } => {
                        // todo ensure lambda matches typed identifier
                        self.parse_lambda(&name, arguments, var_args_start, body)?
                    }
                    exp => {
                        let ext = self.rigz_type(&exp)?;
                        if ext != rigz_type {
                            return Err(ValidationError::InvalidType(format!(
                                "{ext} cannot be assigned to {rigz_type}"
                            )));
                        }
                        self.parse_lazy_expression(exp, &name)?;
                        let var = name.to_string();
                        let mutable = match self.identifiers.entry(name) {
                            Entry::Occupied(mut t) => {
                                let v = t.get();
                                if v.rigz_type == ext {
                                    v.mutable
                                } else {
                                    t.insert(FunctionType {
                                        rigz_type: ext,
                                        mutable,
                                    });
                                    mutable
                                }
                            }
                            Entry::Vacant(v) => {
                                v.insert(FunctionType {
                                    rigz_type: ext,
                                    mutable,
                                });
                                mutable
                            }
                        };
                        if mutable {
                            self.builder.add_load_mut_instruction(var, shadow);
                        } else {
                            self.builder.add_load_let_instruction(var, shadow);
                        }
                    }
                }
            }
            Assign::This => {
                self.mutable_this();
                match expression {
                    Expression::Lambda {
                        arguments,
                        var_args_start,
                        body,
                    } => self.parse_lambda("self", arguments, var_args_start, body)?,
                    exp => {
                        let ext = self.rigz_type(&exp)?;
                        self.identifiers.insert(
                            "self".to_string(),
                            FunctionType {
                                rigz_type: ext,
                                mutable: true,
                            },
                        );
                        self.parse_lazy_expression(exp, "self")?;
                    }
                }
            }
            Assign::Tuple(t) => {
                let rt = self.rigz_type(&expression)?;
                let expt = match rt {
                    RigzType::Tuple(t) => t,
                    _ => vec![RigzType::Any; t.len()],
                };
                // todo support lazy scopes deconstructed into tuples
                self.parse_expression(expression)?;
                for (index, (name, mutable, shadow)) in t.into_iter().enumerate().rev() {
                    let ft = FunctionType {
                        rigz_type: expt[index].clone(),
                        mutable,
                    };
                    let var = name.to_string();
                    self.identifiers.insert(name, ft);
                    self.builder.add_load_instruction((index as i64).into());
                    self.builder.add_instance_get_instruction(index != 0);
                    if mutable {
                        self.builder.add_load_mut_instruction(var, shadow);
                    } else {
                        self.builder.add_load_let_instruction(var, shadow);
                    }
                }
            }
            Assign::InstanceSet(base, calls) => {
                if calls.is_empty() {
                    return Err(ValidationError::MissingExpression(format!(
                        "Invalid InstanceSet call {base:?}"
                    )));
                }
                match base {
                    Expression::This => {
                        self.builder.add_get_self_mut_instruction();
                    }
                    Expression::Identifier(id) => {
                        self.builder.add_get_mutable_variable_instruction(id);
                    }
                    e => {
                        return Err(ValidationError::InvalidType(format!(
                            "Cannot use instance_set for {e:?} - {calls:?}"
                        )))
                    }
                }

                let last = calls.len() - 1;
                let mut calls = calls.into_iter().enumerate();
                let (_, next) = calls.next().unwrap();
                match next {
                    AssignIndex::Identifier(id) => {
                        self.builder.add_load_instruction(id.into());
                    }
                    AssignIndex::Index(index) => {
                        self.parse_expression(index)?;
                    }
                }

                self.parse_expression(expression)?;
                if last > 0 {
                    self.builder.add_instance_get_instruction(true);
                }
                for (i, c) in calls {
                    match c {
                        AssignIndex::Identifier(id) => {
                            self.builder.add_load_instruction(id.into());
                            self.builder.add_instance_get_instruction(i != last);
                        }
                        AssignIndex::Index(index) => {
                            self.parse_expression(index)?;
                            self.builder.add_instance_get_instruction(i != last);
                        }
                    }
                }
                self.builder.add_instance_set_mut_instruction();
            }
        }
        Ok(())
    }

    pub(crate) fn parse_statement(&mut self, statement: Statement) -> Result<(), ValidationError> {
        match statement {
            Statement::Assignment { lhs, expression } => self.parse_assignment(lhs, expression)?,
            Statement::BinaryAssignment {
                lhs: Assign::Identifier { name, .. },
                op,
                expression,
            } => {
                self.builder
                    .add_get_mutable_variable_instruction(name.to_string());
                self.parse_expression(expression)?;
                self.builder.add_binary_assign_instruction(op);
            }
            Statement::BinaryAssignment {
                lhs: Assign::TypedIdentifier { name, .. },
                op,
                expression,
            } => {
                self.builder
                    .add_get_mutable_variable_instruction(name.to_string());
                // todo validate expression is rigz_type
                self.parse_expression(expression)?;
                self.builder.add_binary_assign_instruction(op);
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
                self.mutable_this();
                self.parse_expression(expression)?;
                self.builder.add_binary_assign_instruction(op);
            }
            Statement::Trait(t) => {
                self.parse_trait_definition(t)?;
            }
            Statement::FunctionDefinition(fd) => {
                self.parse_function_definition(fd)?;
            }
            Statement::TypeDefinition(name, def) => {
                self.types.insert(name, def);
            }
            Statement::BinaryAssignment {
                lhs: Assign::Tuple(_),
                op: _,
                expression: _,
            } => {
                return Err(ValidationError::NotImplemented(
                    "Binary assignment not supported for tuple expressions".to_string(),
                ))
            }
            Statement::BinaryAssignment {
                lhs: Assign::InstanceSet(..),
                op: _,
                expression: _,
            } => {
                return Err(ValidationError::NotImplemented(
                    "Binary assignment not supported for InstanceSet".to_string(),
                ))
            }
            Statement::TraitImpl { definitions, .. } => {
                // todo this probably needs some form of checking base_trait and concrete type
                for fd in definitions {
                    self.parse_function_definition(fd)?;
                }
            }
            Statement::ObjectDefinition(definition) => {
                self.parse_object_definition(definition, None)?
            }
            Statement::Enum(e) => {
                if e.variants.iter().map(|v| &v.0).unique().count() != e.variants.len() {
                    return Err(ValidationError::InvalidEnum(format!(
                        "Duplicate variants in {}",
                        e.name
                    )));
                }
                let e = Arc::new(e);
                let index = self.builder.register_enum(e.clone());
                self.enum_lookups.insert(index, e.clone());
                self.enums.insert(e.name.clone(), (index, e));
            }
            Statement::Loop(s) => {
                let in_loop = self.in_loop;
                self.in_loop = true;
                let scope = self.parse_scope(s, "loop")?;
                self.builder.add_loop_instruction(scope);
                self.in_loop = in_loop;
            }
            Statement::For { each, expression, body } => {
                let identifiers = self.identifiers.clone();
                let current = self.builder.current_scope();
                let args = match each {
                    Each::Identifier { name, mutable, shadow } => {
                        self.identifiers.insert(name.clone(), FunctionType {
                            rigz_type: RigzType::Any,
                            mutable,
                        });
                        vec![(name, mutable)]
                    }
                    Each::TypedIdentifier {  name, mutable, shadow, rigz_type } => {
                        self.identifiers.insert(name.clone(), FunctionType {
                            rigz_type,
                            mutable,
                        });
                        vec![(name, mutable)]
                    }
                    Each::Tuple(v) => {
                        let mut res = Vec::with_capacity(v.len());
                        for (name, mutable, shadow) in v {
                            self.identifiers.insert(name.clone(), FunctionType {
                                rigz_type: RigzType::Any,
                                mutable,
                            });
                            res.push((name, mutable))
                        }
                        res
                    }
                };
                let in_loop = self.in_loop;
                self.in_loop = true;
                let new = self.builder.enter_scope("for".to_string(), args, None);
                for element in body.elements {
                    self.parse_element(element)?;
                }
                self.builder.exit_scope(current);
                self.in_loop = in_loop;
                self.parse_expression(expression)?;
                self.builder.add_for_instruction(new);
                self.identifiers = identifiers;
            }
        }
        Ok(())
    }

    fn parse_object_definition(
        &mut self,
        definition: ObjectDefinition,
        dep: Option<usize>,
    ) -> Result<(), ValidationError> {
        let rt = Arc::new(definition.rigz_type);
        let obj = rt.to_string();
        let constructor = match definition.constructor {
            Constructor::Default => {
                let body = Scope {
                    elements: definition
                        .fields
                        .iter()
                        .map(|f| {
                            Element::Statement(Statement::Assignment {
                                lhs: Assign::InstanceSet(
                                    Expression::This,
                                    vec![AssignIndex::Identifier(f.name.clone())],
                                ),
                                expression: Expression::Identifier(f.name.clone()),
                            })
                        })
                        .collect(),
                };
                let args = definition
                    .fields
                    .iter()
                    .map(|a| FunctionArgument {
                        name: a.name.clone(),
                        default: a.default.clone(),
                        function_type: a.attr_type.clone(),
                        var_arg: false,
                        rest: false,
                    })
                    .collect();
                let s = self.parse_constructor(body, rt.clone(), &args)?;
                ObjectConstructor::Scope(args, None, s)
            }
            Constructor::Declaration(args, var) => ObjectConstructor::Custom(args, var),
            Constructor::Definition(args, var, body) => {
                let s = self.parse_constructor(body, rt.clone(), &args)?;
                ObjectConstructor::Scope(args, var, s)
            }
        };

        for func in definition.functions {
            match func {
                FunctionDeclaration::Declaration {
                    name,
                    type_definition,
                } => {
                    let FunctionSignature {
                        arguments,
                        return_type,
                        self_type,
                        var_args_start,
                        arg_type,
                    } = type_definition;
                    let dep = match dep {
                        None => {
                            return Err(ValidationError::InvalidFunction(format!(
                                "Missing object implementation {obj}.{name}"
                            )))
                        }
                        Some(d) => d,
                    };
                    let self_type = if let Some(FunctionType {
                        rigz_type: RigzType::This,
                        mutable,
                    }) = self_type
                    {
                        Some(FunctionType {
                            rigz_type: rt.as_ref().clone(),
                            mutable,
                        })
                    } else {
                        self_type
                    };
                    let fcs = FunctionCallSignature {
                        name: name.clone(),
                        arguments,
                        return_type,
                        self_type,
                        arg_type,
                        var_args_start,
                    };
                    let cs = CallSignature::Function(fcs, CallSite::Object(dep));
                    match self.function_scopes.entry(name) {
                        IndexMapEntry::Occupied(mut ex) => ex.get_mut().push(cs),
                        IndexMapEntry::Vacant(v) => {
                            v.insert(vec![cs]);
                        }
                    };
                }
                FunctionDeclaration::Definition(d) => {
                    let this = match d.type_definition.self_type.as_ref() {
                        None => None,
                        Some(f) => {
                            if f.rigz_type == RigzType::This {
                                let old = self.identifiers.insert(
                                    "self".to_string(),
                                    FunctionType {
                                        rigz_type: rt.as_ref().clone(),
                                        mutable: f.mutable,
                                    },
                                );
                                Some(old)
                            } else {
                                None
                            }
                        }
                    };
                    self.parse_function_definition(d)?;
                    if let Some(old) = this {
                        match old {
                            None => {
                                self.identifiers.remove("self");
                            }
                            Some(old) => {
                                self.identifiers.insert("self".to_string(), old);
                            }
                        }
                    }
                }
            }
        }

        let decl = ObjectDeclaration {
            constructor,
            rigz_type: rt,
            fields: definition.fields,
            dep,
        };
        let old = self.objects.insert(obj, Rc::new(decl));
        if let Some(o) = old {
            warn!("Overwrote previous object {o:?}")
        }
        Ok(())
    }

    fn parse_constructor(
        &mut self,
        body: Scope,
        rigz_type: Arc<RigzType>,
        args: &Vec<FunctionArgument>,
    ) -> Result<usize, ValidationError> {
        let current_vars = self.identifiers.clone();
        let current = self.builder.current_scope();
        self.builder.enter_scope(
            rigz_type.to_string(),
            args.iter()
                .map(|a| (a.name.clone(), a.function_type.mutable))
                .collect(),
            None,
        );
        let res = self.builder.current_scope();
        self.builder
            .add_create_object_instruction(rigz_type.clone());
        self.identifiers.insert(
            "self".to_string(),
            FunctionType {
                rigz_type: rigz_type.as_ref().clone(),
                mutable: true,
            },
        );
        self.builder
            .add_load_mut_instruction("self".to_string(), false);
        for e in body.elements {
            self.parse_element(e)?;
        }
        self.builder.add_get_self_instruction();
        self.builder.exit_scope(current);
        self.identifiers = current_vars;
        Ok(res)
    }

    fn parse_lazy_expression(
        &mut self,
        expression: Expression,
        var: &str,
    ) -> Result<(), ValidationError> {
        match expression {
            Expression::Scope(s) => {
                let scope = self.parse_scope(s, "do")?;
                self.builder.add_load_instruction(LoadValue::ScopeId(scope));
                self.builder.convert_to_lazy_scope(scope, var.to_string());
                Ok(())
            }
            _ => self.parse_expression(expression),
        }
    }

    fn mutable_this(&mut self) {
        self.builder.add_get_self_mut_instruction();
    }

    fn this(&mut self) {
        self.builder.add_get_self_instruction();
    }

    pub(crate) fn parse_function_definition(
        &mut self,
        function_definition: FunctionDefinition,
    ) -> Result<(), ValidationError> {
        let FunctionDefinition {
            name,
            type_definition,
            body,
            lifecycle,
        } = function_definition;
        let identifiers = self.identifiers.clone();
        let type_definition = self.parse_type_signature(&name, type_definition)?;
        let current_scope = self.builder.current_scope();
        let args = type_definition
            .arguments
            .iter()
            .map(|a| (a.name.to_string(), a.function_type.mutable))
            .rev()
            .collect();
        let set_self = type_definition.self_type.as_ref().map(|t| t.mutable);
        let memoized = match lifecycle {
            None => {
                self.builder.enter_scope(name.to_string(), args, set_self);
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
                self.builder
                    .enter_lifecycle_scope(name.to_string(), l, args, set_self);
                memoized
            }
        };
        for arg in &type_definition.arguments {
            let rt = &arg.function_type.rigz_type;
            match rt {
                RigzType::Function(args, ret) => {
                    let args = args.to_vec();
                    let cs = CallSignature::Lambda(type_definition.clone(), args, *ret.clone());
                    match self.function_scopes.entry(arg.name.clone()) {
                        IndexMapEntry::Occupied(mut entry) => {
                            entry.get_mut().push(cs);
                        }
                        IndexMapEntry::Vacant(entry) => {
                            entry.insert(vec![cs]);
                        }
                    }
                }
                _ => {
                    self.identifiers
                        .insert(arg.name.clone(), arg.function_type.clone());
                }
            }
        }
        // todo store arguments variable
        let f_def = self.builder.current_scope();
        let self_type = type_definition.self_type.clone();
        match self.function_scopes.entry(name) {
            IndexMapEntry::Occupied(mut entry) => {
                entry.get_mut().push(CallSignature::Function(
                    type_definition,
                    CallSite::Scope(f_def, memoized),
                ));
            }
            IndexMapEntry::Vacant(e) => {
                e.insert(vec![CallSignature::Function(
                    type_definition,
                    CallSite::Scope(f_def, memoized),
                )]);
            }
        }
        if let Some(t) = &self_type {
            self.identifiers.insert("self".to_string(), t.clone());
        };
        for e in body.elements {
            match e {
                Element::Expression(Expression::This) => match &self_type {
                    Some(t) if t.mutable => {
                        self.mutable_this();
                    }
                    _ => self.parse_element(e)?,
                },
                e => self.parse_element(e)?,
            }
        }
        self.builder.exit_scope(current_scope);
        self.identifiers = identifiers;
        Ok(())
    }

    pub(crate) fn parse_trait_definition_for_module(
        &mut self,
        trait_definition: TraitDefinition,
    ) -> Result<(), ValidationError> {
        let module_name = trait_definition.name;
        for func in trait_definition.functions {
            match func {
                FunctionDeclaration::Declaration {
                    type_definition,
                    name,
                } => {
                    let type_definition = self.parse_type_signature(&name, type_definition)?;
                    match self.function_scopes.entry(name) {
                        IndexMapEntry::Occupied(mut entry) => {
                            entry.get_mut().push(CallSignature::Function(
                                type_definition,
                                CallSite::Module(module_name.to_string()),
                            ));
                        }
                        IndexMapEntry::Vacant(e) => {
                            e.insert(vec![CallSignature::Function(
                                type_definition,
                                CallSite::Module(module_name.to_string()),
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
        name: &str,
        function_signature: FunctionSignature,
    ) -> Result<FunctionCallSignature, ValidationError> {
        let FunctionSignature {
            arguments,
            return_type,
            self_type,
            arg_type,
            var_args_start,
        } = function_signature;
        if self_type.is_none() && return_type.mutable {
            return Err(ValidationError::InvalidFunction(
                "Cannot have mutable return type on non-extension function".to_string(),
            ));
        };

        Ok(FunctionCallSignature {
            name: name.to_string(),
            arguments,
            return_type,
            self_type,
            arg_type,
            var_args_start,
        })
    }

    pub(crate) fn parse_trait_definition(
        &mut self,
        trait_definition: TraitDefinition,
    ) -> Result<(), ValidationError> {
        for func in trait_definition.functions {
            match func {
                FunctionDeclaration::Declaration { .. } => {
                    // todo currently handled in impl statement, I'm sure there are some cases that should work that don't
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
                        Some(m.definition.name.clone())
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
        expression: Expression,
    ) -> Result<(), ValidationError> {
        match expression {
            Expression::DoubleBang(e) => {
                self.parse_expression(*e)?;
                // todo should this be a return if error instead?
                self.builder.add_halt_if_error_instruction();
            }
            Expression::This => {
                self.this();
            }
            Expression::Break => {
                if !self.in_loop {
                    return Err(ValidationError::InvalidType("break cannot be used outside of loop".to_string()))
                }
                self.builder.add_break_instruction();
            }
            Expression::Next => {
                if !self.in_loop {
                    return Err(ValidationError::InvalidType("next cannot be used outside of loop".to_string()))
                }
                self.builder.add_next_instruction();
            }
            Expression::Error(e) => {
                self.parse_expression(*e)?;
                self.builder.add_cast_instruction(RigzType::Error);
                self.builder.add_ret_instruction();
            }
            Expression::Tuple(v) => {
                self.parse_tuple(v)?;
            }
            Expression::Index(base, index) => {
                self.parse_expression(*base)?;
                self.parse_expression(*index)?;
                self.builder.add_instance_get_instruction(false);
            }
            Expression::Value(v) => self.parse_value(v.into()),
            Expression::BinExp(a, op, b) => {
                self.parse_expression(*a)?;
                self.parse_expression(*b)?;
                self.builder.add_binary_instruction(op);
            }
            Expression::UnaryExp(op, ex) => {
                self.parse_expression(*ex)?;
                self.builder.add_unary_instruction(op);
            }
            Expression::Identifier(id) => {
                if self.function_scopes.contains_key(&id) {
                    self.call_function(None, &id, vec![].into())?;
                } else {
                    self.builder.add_get_variable_instruction(id.to_string());
                }
            }
            Expression::If {
                condition,
                then,
                branch,
            } => {
                self.parse_expression(*condition)?;
                let if_output = self.parse_scope(then, "if")?;
                match branch {
                    None => {
                        self.builder.add_if_instruction(if_output);
                    }
                    Some(p) => {
                        let else_output = self.parse_scope(p, "else")?;
                        self.builder.add_if_else_instruction(if_output, else_output);
                    }
                }
            }
            Expression::Unless { condition, then } => {
                self.parse_expression(*condition)?;
                let unless = self.parse_scope(then, "unless")?;
                self.builder.add_unless_instruction(unless);
            }
            Expression::List(list) => {
                self.parse_list(list)?;
            }
            Expression::Map(map) => {
                self.parse_map(map)?;
            }
            Expression::Function(f) => {
                self.parse_function(f)?;
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
                let old = self
                    .identifiers
                    .insert(var.clone(), FunctionType::new(RigzType::Any));
                let inner_scope = self.builder.enter_scope(
                    "for-list".to_string(),
                    vec![(var.to_string(), false)],
                    None,
                );
                self.parse_expression(*body)?;
                self.builder.exit_scope(current);
                match old {
                    None => {
                        self.identifiers.remove(&var);
                    }
                    Some(t) => {
                        *self.identifiers.get_mut(&var).unwrap() = t;
                    }
                }
                self.parse_expression(*exp)?;
                self.builder.add_for_list_instruction(inner_scope);
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
                let k_old = self
                    .identifiers
                    .insert(k_var.clone(), FunctionType::new(RigzType::Any));
                let v_old = self
                    .identifiers
                    .insert(v_var.clone(), FunctionType::new(RigzType::Any));
                let inner_scope = self.builder.enter_scope(
                    "for-map".to_string(),
                    vec![(k_var.to_string(), false), (v_var.to_string(), false)],
                    None,
                );
                match value {
                    None => {
                        self.parse_expression(*key)?;
                    }
                    Some(value) => {
                        self.parse_tuple(vec![*key, *value])?;
                    }
                }
                self.builder.exit_scope(current);
                match k_old {
                    None => {
                        self.identifiers.remove(&k_var);
                    }
                    Some(t) => {
                        *self.identifiers.get_mut(&k_var).unwrap() = t;
                    }
                }
                match v_old {
                    None => {
                        self.identifiers.remove(&v_var);
                    }
                    Some(t) => {
                        *self.identifiers.get_mut(&v_var).unwrap() = t;
                    }
                }
                self.parse_expression(*expression)?;
                self.builder.add_for_map_instruction(inner_scope);
            }
            Expression::Scope(s) => {
                let s = self.parse_scope(s, "do")?;
                self.builder.add_load_instruction(LoadValue::ScopeId(s));
            }
            Expression::Cast(e, t) => {
                self.parse_expression(*e)?;
                self.builder.add_cast_instruction(t);
            }
            Expression::Symbol(s) => {
                let index = self.find_or_create_constant(s.into());
                self.builder
                    .add_load_instruction(LoadValue::Constant(index));
            }
            Expression::Return(ret) => {
                match ret {
                    None => {
                        let none = self.find_or_create_constant(ObjectValue::default());
                        self.builder.add_load_instruction(LoadValue::Constant(none));
                    }
                    Some(e) => {
                        self.parse_expression(*e)?;
                    }
                };
                self.builder.add_ret_instruction();
            }
            Expression::Into { base, next } => {
                self.parse_function(next.prepend(*base))?;
            }
            Expression::Try(b) => {
                if let Expression::Catch { .. } = b.as_ref() {
                    warn!("try is ignored with catch");
                    return self.parse_expression(*b);
                }
                self.parse_expression(*b)?;
                self.builder.add_try_instruction();
            }
            Expression::Catch { base, var, catch } => {
                if let Expression::Try(b) = *base {
                    warn!("try is ignored with catch");
                    return self.parse_expression(Expression::Catch {
                        base: b,
                        var,
                        catch,
                    });
                }
                self.parse_expression(*base)?;
                let old = var.as_ref().map(|v| self.identifiers.remove_entry(v));
                var.as_ref().map(|v| {
                    self.identifiers
                        .insert(v.clone(), FunctionType::new(RigzType::Any))
                });
                let current = self.builder.current_scope();
                let has_arg = var.is_some();
                let inner = self.builder.enter_scope(
                    "catch".to_string(),
                    var.map(|s| vec![(s, false)]).unwrap_or(vec![]),
                    None,
                );
                for e in catch.elements {
                    self.parse_element(e)?;
                }
                self.builder.exit_scope(current);
                old.map(|v| v.map(|(k, v)| self.identifiers.insert(k, v)));
                self.builder.add_catch_instruction(inner, has_arg);
            }
            Expression::Match {
                condition,
                variants,
            } => {
                let rt = self.rigz_type(&condition)?;
                let var = variants.len();
                let base = match rt {
                    RigzType::Enum(i) => self.enum_lookups.get(&i).cloned(),
                    _ => None,
                };
                let mut match_arms = vec![];
                for (index, v) in variants.into_iter().enumerate() {
                    match (&base, v) {
                        (None, MatchVariant::Enum { name, .. }) => {
                            return Err(ValidationError::InvalidEnum(format!(
                                "Unknown enum match statement .{name} for {condition:?} ({rt:?})"
                            )))
                        }
                        (
                            Some(en),
                            MatchVariant::Enum {
                                name,
                                condition: cond,
                                body,
                                variables,
                            },
                        ) => {
                            match en.variants.iter().find_position(|(v, vt)| v == &name) {
                                None => {
                                    return Err(ValidationError::InvalidEnum(format!(
                                    "Illegal enum match variant .{name} for {condition:?} ({rt:?})"
                                )))
                                }
                                Some((vi, (vname, vt))) => {
                                    match cond {
                                        MatchVariantCondition::None => {
                                            let scope = self.parse_scope(body, vname)?;
                                            match_arms.push(MatchArm::Enum(vi, scope));
                                        }
                                        // Todo all expressions will need to be processed or they'll hold over on stack
                                        MatchVariantCondition::If(ex) => {
                                            self.parse_expression(ex)?;
                                            let scope = self.parse_scope(body, vname)?;
                                            match_arms.push(MatchArm::If(vi, scope));
                                        }
                                        MatchVariantCondition::Unless(ex) => {
                                            self.parse_expression(ex)?;
                                            let scope = self.parse_scope(body, vname)?;
                                            match_arms.push(MatchArm::Unless(vi, scope));
                                        }
                                    };
                                }
                            }
                        }
                        (_, MatchVariant::Else(scope)) => {
                            if index + 1 != var {
                                warn!(
                                    "else arm should be last, remaining match arms will be skipped"
                                );
                            }
                            let scope = self.parse_scope(scope, "else")?;
                            match_arms.push(MatchArm::Else(scope));
                        }
                    }
                }
                self.parse_expression(*condition)?;
                self.builder.add_match_instruction(match_arms);
            }
            Expression::Enum(t, v, ex) => {
                let (e_index, e) = match self.enums.get(&t) {
                    None => {
                        return Err(ValidationError::InvalidEnum(format!("{t} does not exist")))
                    }
                    Some((e_index, e)) => (*e_index, e.clone()),
                };
                let pos = e.variants.iter().find_position(|(e, _)| e == &v);
                let (index, ty) = match pos {
                    None => {
                        return Err(ValidationError::InvalidEnum(format!(
                            "{t}.{v} does not exist"
                        )))
                    }
                    Some((i, v)) => (i, &v.1),
                };
                // todo type checking
                let has_value = match (ty, ex) {
                    (RigzType::None, None) => false,
                    (RigzType::None, Some(e)) => {
                        return Err(ValidationError::InvalidEnum(format!(
                            "{t}.{v}, expected no arguments, received {e:?}"
                        )))
                    }
                    (
                        RigzType::Wrapper {
                            base_type,
                            optional,
                            can_return_error,
                        },
                        ex,
                    ) if *optional => match ex {
                        None => false,
                        Some(e) => {
                            self.parse_expression(*e)?;
                            true
                        }
                    },
                    (rt, None) => {
                        return Err(ValidationError::InvalidEnum(format!(
                            "{t}.{v}, expected {rt:?}, received none"
                        )))
                    }
                    (rt, Some(e)) => {
                        self.parse_expression(*e)?;
                        true
                    }
                };
                self.builder
                    .add_create_enum_instruction(e_index, index, has_value);
            }
        }
        Ok(())
    }

    fn parse_function(
        &mut self,
        function_expression: FunctionExpression,
    ) -> Result<(), ValidationError> {
        match function_expression {
            FunctionExpression::FunctionCall(name, args) => {
                self.call_function(None, &name, args)?;
            }
            // todo make a clear delineation between self.foo & Self.foo
            FunctionExpression::TypeFunctionCall(rigz_type, name, args) => {
                self.call_function(Some(rigz_type), &name, args)?;
            }
            FunctionExpression::InstanceFunctionCall(exp, calls, args) => {
                let len = calls.len();
                assert!(len > 0, "Invalid Instance Function Call no calls");
                let last = len - 1;
                let mut calls = calls.into_iter().enumerate();
                let (_, first) = calls.next().unwrap();
                self.check_module_exists(&first)?;
                match self.function_scopes.contains_key(&first) {
                    false => {
                        self.parse_expression(*exp)?;
                        self.builder.add_load_instruction(first.into());
                        self.builder.add_instance_get_instruction(false);
                    }
                    true if last == 0 => {
                        self.call_extension_function(*exp, &first, args)?;
                        return Ok(());
                    }
                    true => {
                        self.call_extension_function(*exp, &first, vec![].into())?;
                    }
                };

                for (index, c) in calls {
                    let fcs = match self.function_scopes.get(&c) {
                        None => {
                            self.builder.add_load_instruction(c.into());
                            self.builder.add_instance_get_instruction(false);
                            continue;
                        }
                        Some(fcs) => fcs.clone(),
                    };

                    if index == last {
                        self.call_inline_extension(&c, fcs, args)?;
                        break;
                    } else {
                        self.call_inline_extension(&c, fcs, vec![].into())?;
                    }
                }
            }
            FunctionExpression::TypeConstructor(ty, args) => {
                let ty = ty.to_string();
                let dec = match self.objects.get(&ty) {
                    None => {
                        return Err(ValidationError::InvalidType(format!(
                            "Missing constructor for {ty}"
                        )))
                    }
                    Some(dec) => dec.clone(),
                };
                let (cargs, var, scope) = match &dec.constructor {
                    ObjectConstructor::Scope(cargs, var, s) => {
                        (cargs.clone(), var.clone(), Some(*s))
                    }
                    ObjectConstructor::Custom(cargs, var) => (cargs.clone(), var.clone(), None),
                };

                let args = self.setup_call_args(
                    args,
                    FunctionCallSignature {
                        name: "Self".to_string(),
                        arguments: cargs,
                        return_type: FunctionType {
                            rigz_type: Default::default(),
                            mutable: false,
                        },
                        self_type: None,
                        arg_type: ArgType::Positional,
                        var_args_start: var,
                    },
                )?;

                match scope {
                    None => match dec.dep {
                        None => {
                            return Err(ValidationError::InvalidType(format!(
                                "{ty} is not a Custom Type, definition required for object"
                            )))
                        }
                        Some(d) => {
                            self.builder.add_call_dependency_instruction(args, d);
                        }
                    },
                    Some(s) => {
                        self.builder.add_call_instruction(s);
                    }
                }
            }
        }
        Ok(())
    }

    fn find_or_create_constant(&mut self, value: ObjectValue) -> usize {
        match self.constants.entry(value) {
            IndexMapEntry::Occupied(e) => *e.get(),
            IndexMapEntry::Vacant(e) => {
                let index = self.builder.add_constant(e.key().clone());
                e.insert(index);
                index
            }
        }
    }

    fn call_inline_extension(
        &mut self,
        name: &str,
        function_call_signatures: FunctionCallSignatures,
        args: RigzArguments,
    ) -> Result<(), ValidationError> {
        if function_call_signatures.len() == 1 {
            let mut fcs = function_call_signatures.into_iter();
            match fcs.next().unwrap() {
                CallSignature::Function(fcs, call) => {
                    let (vm_module, mutable) = match &fcs.self_type {
                        None => {
                            return Err(ValidationError::InvalidFunction(format!(
                                "Function is not an extension function {name}"
                            )))
                        }
                        Some(this) => {
                            let is_vm = this.rigz_type.is_vm();
                            (is_vm, this.mutable)
                        }
                    };
                    let len = self.setup_call_args(args, fcs)?;
                    self.process_extension_call(name.to_string(), vm_module, mutable, len, call);
                }
                CallSignature::Lambda(..) => {
                    return Err(ValidationError::InvalidFunction(format!(
                        "extension lambdas are not supported {name}"
                    )))
                }
            };
        } else {
            return Err(ValidationError::NotImplemented(format!(
                "Multiple extension functions match for {name}"
            )));
        }
        Ok(())
    }

    fn parse_list(&mut self, list: Vec<Expression>) -> Result<(), ValidationError> {
        let mut base = Vec::new();
        let mut values_only = true;
        for (index, v) in list.into_iter().enumerate() {
            if values_only {
                match v {
                    Expression::Value(v) => {
                        base.push(v.into());
                    }
                    e => {
                        values_only = false;
                        let index = Number::Int(index as i64);
                        self.builder
                            .add_load_instruction(ObjectValue::List(base.into()).into());
                        base = Vec::new();
                        self.builder.add_load_instruction(index.into());
                        self.parse_expression(e)?;
                        self.builder.add_instance_set_instruction();
                    }
                }
            } else {
                let index = Number::Int(index as i64);
                self.builder.add_load_instruction(index.into());
                self.parse_expression(v)?;
                self.builder.add_instance_set_instruction();
            }
        }
        if values_only {
            self.builder
                .add_load_instruction(ObjectValue::List(base).into());
        }
        Ok(())
    }

    fn parse_tuple(&mut self, list: Vec<Expression>) -> Result<(), ValidationError> {
        let mut base = Vec::new();
        let mut values_only = true;
        for (index, v) in list.into_iter().enumerate() {
            if values_only {
                match v {
                    Expression::Value(v) => {
                        base.push(v.into());
                    }
                    e => {
                        values_only = false;
                        self.builder
                            .add_load_instruction(ObjectValue::Tuple(base.into()).into());
                        base = Vec::new();
                        let index = Number::Int(index as i64);
                        self.builder.add_load_instruction(index.into());
                        self.parse_expression(e)?;
                        self.builder.add_instance_set_instruction();
                    }
                }
            } else {
                let index = Number::Int(index as i64);
                self.builder.add_load_instruction(index.into());
                self.parse_expression(v)?;
                self.builder.add_instance_set_instruction();
            }
        }
        if values_only {
            self.builder
                .add_load_instruction(ObjectValue::Tuple(base).into());
        }
        Ok(())
    }

    fn parse_map(&mut self, map: Vec<(Expression, Expression)>) -> Result<(), ValidationError> {
        let mut base = IndexMap::new();
        let mut values_only = true;

        for (k, v) in map {
            if values_only {
                match (k, v) {
                    (Expression::Value(k), Expression::Value(v)) => {
                        base.insert(k.into(), v.into());
                    }
                    (Expression::Identifier(k), Expression::Value(v)) => {
                        base.insert(k.to_string().into(), v.into());
                    }
                    (Expression::Identifier(k), e) => {
                        values_only = false;
                        self.builder
                            .add_load_instruction(ObjectValue::Map(base).into());
                        self.builder.add_load_instruction(k.into());
                        self.parse_expression(e)?;
                        self.builder.add_instance_set_instruction();
                        base = IndexMap::new();
                    }
                    (k, v) => {
                        values_only = false;
                        self.builder
                            .add_load_instruction(ObjectValue::Map(base).into());
                        base = IndexMap::new();
                        self.parse_expression(k)?;
                        self.parse_expression(v)?;
                        self.builder.add_instance_set_instruction();
                    }
                }
            } else {
                match (k, v) {
                    (Expression::Identifier(k), e) => {
                        self.builder.add_load_instruction(k.into());
                        self.parse_expression(e)?;
                        self.builder.add_instance_set_instruction();
                    }
                    (k, v) => {
                        self.parse_expression(k)?;
                        self.parse_expression(v)?;
                        self.builder.add_instance_set_instruction();
                    }
                }
            }
        }

        if values_only {
            self.builder
                .add_load_instruction(ObjectValue::Map(base).into());
        }

        Ok(())
    }

    fn str_to_log_level(raw: &str) -> Result<Level, ValidationError> {
        let level = match raw.to_lowercase().as_str() {
            "info" => Level::Info,
            "warn" => Level::Warn,
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

    fn call_built_in_function(
        &mut self,
        name: &str,
        arguments: RigzArguments,
    ) -> Result<Option<RigzArguments>, ValidationError> {
        let RigzArguments::Positional(arguments) = arguments else {
            return Ok(Some(arguments));
        };

        match name {
            "puts" => {
                let len = arguments.len();
                for arg in arguments.into_iter().rev() {
                    self.parse_expression(arg)?;
                }
                self.builder.add_puts_instruction(len);
            }
            "log" => {
                if arguments.len() >= 2 {
                    let len = arguments.len() - 2;
                    let mut arguments = arguments.iter();
                    let level = match arguments.next().unwrap() {
                        Expression::Value(PrimitiveValue::String(s)) => {
                            Self::str_to_log_level(s.as_str())?
                        }
                        Expression::Symbol(s) => Self::str_to_log_level(s)?,
                        // todo support identifiers here
                        e => {
                            return Err(ValidationError::InvalidFunction(format!(
                                "Unable to create log level for {e:?}, must be string or symbol"
                            )))
                        }
                    };

                    let template = match arguments.next().unwrap() {
                        Expression::Value(PrimitiveValue::String(s)) => s.clone(),
                        _ => "{}".to_string(),
                    };

                    for arg in arguments {
                        self.parse_expression(arg.clone())?;
                    }

                    self.builder.add_log_instruction(level, template, len);
                } else {
                    return Err(ValidationError::InvalidFunction(format!(
                        "Invalid args to log, need at least 2 arguments - {arguments:?}"
                    )));
                }
            }
            "send" => {
                if arguments.is_empty() {
                    return Err(ValidationError::InvalidFunction("`send` requires at least one argument that includes the event being triggered".to_string()));
                }
                let args = arguments.len();
                for e in arguments.into_iter().rev() {
                    self.parse_expression(e)?;
                }
                self.builder.add_send_instruction(args);
            }
            "receive" => {
                let args = arguments.len();
                if matches!(args, 1 | 2) {
                    for e in arguments.into_iter().rev() {
                        self.parse_expression(e)?;
                    }
                    self.builder.add_receive_instruction(args);
                } else {
                    return Err(ValidationError::InvalidFunction(format!("Invalid args to `receive`, only possible arguments are process_id (required) and timeout (ms, optional defaults to infinity) - {arguments:?}")));
                }
            }
            "spawn" => {
                let len = arguments.len();
                let mut args = arguments.into_iter();
                let (scope_id, timeout) = match len {
                    1 => {
                        let scope = args.next().unwrap();
                        let id = match scope {
                            Expression::Scope(s) => self.parse_scope(s, "spawn")?,
                            _ => {
                                return Err(ValidationError::NotImplemented(format!(
                                    "Only scopes are supported for `spawn` - receieved {scope:?}"
                                )))
                            }
                        };
                        (id, false)
                    }
                    2 => {
                        let timeout = args.next().unwrap();
                        self.parse_expression(timeout)?;
                        let scope = args.next().unwrap();
                        let id = match scope {
                            Expression::Scope(s) => self.parse_scope(s, "spawn")?,
                            _ => {
                                return Err(ValidationError::NotImplemented(format!(
                                    "Only scopes are supported for `spawn` - receieved {scope:?}"
                                )))
                            }
                        };
                        (id, true)
                    }
                    _ => {
                        return Err(ValidationError::InvalidFunction("`spawn` requires the scope argument to initialize the process with an optional timeout (ms), i.e. `spawn do = 'hi'` or `spawn 1, do = 42`".to_string()));
                    }
                };
                self.builder.add_spawn_instruction(scope_id, timeout);
            }
            "sleep" => {
                if arguments.len() != 1 {
                    return Err(ValidationError::InvalidFunction(
                        "`sleep` requires one argument, duration to sleep (ms)".to_string(),
                    ));
                }
                let mut args = arguments.into_iter();
                self.parse_expression(args.next().unwrap())?;
                self.builder.add_sleep_instruction();
            }
            _ => return Ok(Some(RigzArguments::Positional(arguments))),
        }
        Ok(None)
    }

    fn call_function(
        &mut self,
        rigz_type: Option<RigzType>,
        name: &str,
        arguments: RigzArguments,
    ) -> Result<(), ValidationError> {
        let Some(arguments) = self.call_built_in_function(name, arguments)? else {
            return Ok(());
        };

        if arguments.is_empty() {
            if let Some(v) = self.identifiers.get(name) {
                if v.mutable {
                    self.builder
                        .add_get_mutable_variable_instruction(name.to_string());
                } else {
                    self.builder.add_get_variable_instruction(name.to_string());
                }
                return Ok(());
            }
        }

        self.check_module_exists(name)?;

        let BestMatch {
            fcs,
            mutable: _,
            vm_module,
        } = self.best_matched_function(name, rigz_type, &arguments)?;

        match fcs {
            CallSignature::Function(fcs, call) => {
                let len = self.setup_call_args(arguments, fcs)?;
                match call {
                    CallSite::Scope(s, memo) => {
                        if memo {
                            self.builder.add_call_memo_instruction(s);
                        } else {
                            self.builder.add_call_instruction(s);
                        }
                    }
                    CallSite::Module(m) => {
                        if vm_module {
                            // self.builder.add_call_vm_extension_module_instruction(
                            //     m,
                            //     name.to_string(),
                            //     len,
                            // );
                        } else {
                            self.builder
                                .add_call_module_instruction(m, name.to_string(), len);
                        }
                    }
                    CallSite::Object(dep) => {
                        self.builder
                            .add_call_object_instruction(dep, name.to_string(), len);
                    }
                }
            }
            CallSignature::Lambda(_fcs, args, _) => {
                let arguments = if let RigzArguments::Positional(a) = arguments {
                    a
                } else {
                    return Err(ValidationError::NotImplemented(format!(
                        "Non-positional args not supported for lambdas - {name}"
                    )));
                };
                for (_arg, actual) in args.iter().zip(arguments) {
                    // todo ensure arg matches actual
                    self.parse_expression(actual)?;
                }
                self.builder.add_get_variable_instruction(name.to_string());
            }
        };
        Ok(())
    }

    fn best_matched_function(
        &self,
        name: &str,
        rigz_type: Option<RigzType>,
        arguments: &RigzArguments,
    ) -> Result<BestMatch, ValidationError> {
        let mut fcs = None;
        let mut vm_module = false;
        let mut mutable = false;

        let function_call_signatures = self.get_function(name)?;
        if function_call_signatures.len() == 1 {
            match &function_call_signatures[0] {
                CallSignature::Function(inner_fcs, call_site) => {
                    let arguments = inner_fcs.convert_ref(arguments);
                    if arguments.len() <= inner_fcs.arguments.len() {
                        match &inner_fcs.self_type {
                            None => {}
                            Some(ft) => {
                                vm_module = ft.rigz_type.is_vm();
                                mutable = ft.mutable;
                            }
                        }
                        fcs = Some(CallSignature::Function(
                            inner_fcs.clone(),
                            call_site.clone(),
                        ));
                    } else if inner_fcs.var_args_start.is_none() {
                        return Err(ValidationError::InvalidFunction(format!(
                            "Expected function with var_args {name}"
                        )));
                    } else {
                        // var args
                        fcs = Some(CallSignature::Function(
                            inner_fcs.clone(),
                            call_site.clone(),
                        ));
                    }
                }
                lambda => fcs = Some(lambda.clone()),
            }
        } else {
            let arg_len = arguments.len();

            for cs in function_call_signatures {
                match cs {
                    CallSignature::Function(fc, call_site) => {
                        // let arguments = fc.convert_ref(arguments); todo check arg type
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
                            (Some(ft), Some(s)) => {
                                let s = if let RigzType::Wrapper { base_type, .. } = s {
                                    base_type.as_ref()
                                } else {
                                    s
                                };
                                if &ft.rigz_type == s {
                                    if arg_len <= fc_arg_len {
                                        vm_module = ft.rigz_type.is_vm();
                                        mutable = ft.mutable;
                                        fcs = Some(CallSignature::Function(fc, call_site));
                                        break;
                                    } else {
                                        match fc.var_args_start {
                                            None => {}
                                            Some(i) => {
                                                if (fc_arg_len - 1) % (arg_len - i) == 0 {
                                                    mutable = ft.mutable;
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
                    CallSignature::Lambda(acs, args, ret) => {
                        if rigz_type.is_none() && args.len() == arg_len {
                            fcs = Some(CallSignature::Lambda(acs, args, ret));
                            break;
                        }
                    }
                }
            }
        }
        // todo support runtime function matching?
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
            }),
        }
    }

    fn setup_call_args(
        &mut self,
        arguments: RigzArguments,
        fcs: FunctionCallSignature, // todo don't use FCS here, create a minimal type
    ) -> Result<usize, ValidationError> {
        let arguments = fcs.convert(arguments)?;
        let al = arguments.len();
        let arguments = if al < fcs.arguments.len() {
            let mut arguments = arguments;
            let (_, rem) = fcs.arguments.split_at(al);
            for arg in rem {
                match &arg.default {
                    None => {
                        return Err(ValidationError::MissingExpression(format!(
                            "Invalid args for {} expected default value for {arg:?}",
                            fcs.name
                        )));
                    }
                    Some(e) => arguments.push(e.clone()),
                }
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
                            match arg.default.as_ref() {
                                None => {
                                    return Err(ValidationError::MissingExpression(format!(
                                        "Invalid var_args for {} expected default value for {arg:?}",
                                        fcs.name
                                    )));
                                }
                                Some(e) => a[index + last_var_arg].push(e.clone()),
                            }
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
        let arg_len = arguments.len();
        for (arg, expression) in fcs.arguments.iter().zip(arguments) {
            match expression {
                Expression::Lambda {
                    arguments,
                    var_args_start,
                    body,
                } => {
                    self.parse_anon_lambda(&fcs, &arg.name, arguments, var_args_start, *body)?;
                }
                _ => {
                    if let RigzType::Function(args, res) = &arg.function_type.rigz_type {
                        let Expression::Identifier(id) = expression else {
                            return Err(ValidationError::InvalidFunction(format!("Function type argument, expected anonymous lambda or function reference |{args:?}| -> {res}, received {expression:?}")));
                        };
                        match self.function_scopes.get(&id) {
                            None => {
                                self.builder
                                    .add_get_variable_reference_instruction(arg.name.to_string());
                            }
                            Some(fcs) => {
                                let func = fcs.iter()
                                    .filter_map(|cs| match cs {
                                        CallSignature::Function(fcs, cs) => {
                                            match cs {
                                                // todo better matching on args, support defaults
                                                CallSite::Scope(id, _) if fcs.arguments.len() == args.len() => {
                                                    Some(*id)
                                                }
                                                CallSite::Module(m) => {
                                                    error!("Module function references are not supported yet {id} - Module {m}");
                                                    None
                                                },
                                                _ => None,
                                            }
                                        }
                                        CallSignature::Lambda(_, _, _) => None,
                                    })
                                    .collect::<Vec<_>>();
                                if func.is_empty() {
                                    self.builder.add_get_variable_reference_instruction(
                                        arg.name.to_string(),
                                    );
                                    continue;
                                }
                                if func.len() > 1 {
                                    return Err(ValidationError::InvalidFunction(format!(
                                        "Ambiguous function reference {id} - {func:?}"
                                    )));
                                }
                                let func = func[0];
                                self.builder.add_load_instruction(LoadValue::ScopeId(func));
                                self.builder
                                    .add_load_let_instruction(arg.name.to_string(), false);
                                self.builder
                                    .add_get_variable_reference_instruction(arg.name.to_string());
                            }
                        }
                    } else {
                        self.parse_expression(expression)?;
                    }
                }
            }
        }
        Ok(arg_len)
    }

    fn call_extension_function(
        &mut self,
        this_exp: Expression,
        name: &str,
        arguments: RigzArguments,
    ) -> Result<(), ValidationError> {
        if let Expression::Lambda { .. } = this_exp {
            return Err(ValidationError::InvalidFunction("Cannot call function on lambda, use {{ || <expression> }} or do || end syntax instead when chaining".to_string()));
        }

        let rigz_type = self.rigz_type(&this_exp)?;
        let BestMatch {
            fcs,
            mutable,
            vm_module,
        } = self.best_matched_function(name, Some(rigz_type), &arguments)?;
        match fcs {
            CallSignature::Function(fcs, call) => {
                let len = self.setup_call_args(arguments, fcs)?;
                self.parse_extension_expression(mutable, this_exp)?;
                self.process_extension_call(name.to_string(), vm_module, mutable, len, call);
            }
            CallSignature::Lambda(..) => {
                return Err(ValidationError::InvalidFunction(format!(
                    "extension lambdas are not supported {name}"
                )))
            }
        };
        Ok(())
    }

    fn process_extension_call(
        &mut self,
        name: String,
        vm_module: bool,
        mutable: bool,
        args: usize,
        call: CallSite,
    ) {
        match call {
            CallSite::Scope(s, memo) => {
                if memo {
                    self.builder.add_call_memo_instruction(s);
                } else {
                    self.builder.add_call_instruction(s);
                }
            }
            CallSite::Module(m) => {
                if vm_module {
                    // self.builder
                    //     .add_call_vm_extension_module_instruction(m, name, args);
                } else if mutable {
                    self.builder
                        .add_call_mutable_extension_module_instruction(m, name, args);
                } else {
                    self.builder
                        .add_call_extension_module_instruction(m, name, args);
                }
            }
            CallSite::Object(_) => {
                if mutable {
                    self.builder
                        .add_call_mutable_object_extension_module_instruction(name, args);
                } else {
                    self.builder
                        .add_call_extension_object_instruction(name, args);
                }
            }
        }
    }

    fn parse_extension_expression(
        &mut self,
        mutable: bool,
        expression: Expression,
    ) -> Result<(), ValidationError> {
        match expression {
            Expression::Identifier(id) => {
                let id = id.to_string();
                if mutable {
                    self.builder.add_get_mutable_variable_instruction(id);
                } else {
                    self.builder.add_get_variable_instruction(id);
                }
            }
            _ => {
                self.parse_expression(expression)?;
            }
        }
        Ok(())
    }

    fn parse_contents<P: Debug>(
        &mut self,
        contents: String,
        path: P,
    ) -> Result<usize, ValidationError> {
        let input = contents.as_str();
        let parser = match Parser::prepare(input, self.parser_options.clone()) {
            Ok(p) => p,
            Err(e) => {
                return Err(ValidationError::InvalidImport(format!(
                    "Failed to setup parser {path:?} - {e}"
                )))
            }
        };

        let program = match parser.parse() {
            Ok(p) => p.into(),
            Err(e) => {
                return Err(ValidationError::InvalidImport(format!(
                    "Failed to parse {path:?} - {e}"
                )))
            }
        };
        let current = self.builder.current_scope();
        let dest = self.builder.enter_scope(format!("{path:?}"), vec![], None);
        // skip validation, imports don't need to end with an expression
        if let Err(e) = self.parse_scoped_program(program, Some(current)) {
            return Err(ValidationError::InvalidImport(format!(
                "Failed to process {path:?} - {e}"
            )));
        }

        Ok(dest)
    }

    fn parse_file(&mut self, parse: &PathBuf) -> Result<usize, ValidationError> {
        let raw = match std::fs::read_to_string(parse) {
            Ok(s) => s,
            Err(e) => {
                return Err(ValidationError::InvalidImport(format!(
                    "Failed to read {parse:?} - {e}"
                )))
            }
        };
        self.parse_contents(raw, parse)
    }

    fn download(&self, url: &str) -> Result<String, ValidationError> {
        match ureq::get(url).call().map(|r| r.into_string()) {
            Ok(Ok(s)) => Ok(s),
            Ok(Err(e)) => Err(ValidationError::DownloadFailed(format!(
                "Failed to parse response {url} - {e}"
            ))),
            Err(e) => Err(ValidationError::DownloadFailed(format!(
                "Failed to download {url} - {e}"
            ))),
        }
    }

    fn parse_url(&mut self, url: &str) -> Result<usize, ValidationError> {
        let contents = self.download(url)?;
        self.parse_contents(contents, url)
    }

    fn parse_import_path(&mut self, import_path: ImportPath) -> Result<(), ValidationError> {
        let root = match &import_path {
            ImportPath::Url(url) => self.parse_url(url),
            ImportPath::File(path) => self.parse_file(path),
        }?;
        self.builder.add_call_instruction(root);
        self.imports.insert(import_path, Imports { root });
        Ok(())
    }

    fn parse_import(&mut self, import: ImportValue) -> Result<(), ValidationError> {
        let name = match import {
            ImportValue::TypeValue(tv) => tv,
            ImportValue::FilePath(f) => {
                if self.parser_options.current_directory.is_none() {
                    self.parser_options.current_directory = match env::current_dir() {
                        Ok(f) => {
                            println!("Current Dir: {f:?}");
                            Some(f)
                        }
                        Err(e) => {
                            return Err(ValidationError::InvalidImport(format!(
                                "Failed to get current directory - {e}"
                            )))
                        }
                    }
                }
                let parse = match &self.parser_options.current_directory {
                    None => {
                        return Err(ValidationError::InvalidImport(format!(
                            "Current Directory is not set, unable to parse_file {f}"
                        )))
                    }
                    Some(p) => ImportPath::File(p.join(&f)),
                };
                if self.imports.contains_key(&parse) {
                    return Ok(());
                }
                return self.parse_import_path(parse);
            }
            ImportValue::UrlPath(url) => {
                let path = ImportPath::Url(url);
                if self.imports.contains_key(&path) {
                    return Ok(());
                }

                return self.parse_import_path(path);
            } // todo support `import "<URL | path>" as foo`
              // todo support `import dep` to support external resources, like package.json or Gemfile
        };

        match self.modules.get_mut(name.as_str()) {
            None => {
                // todo support non module imports
                return Err(ValidationError::ModuleError(format!(
                    "Module {name} does not exist"
                )));
            }
            Some(def) => {
                if let ModuleDefinition::Module(_) = def {
                    let ModuleDefinition::Module(def) =
                        std::mem::replace(def, ModuleDefinition::Imported)
                    else {
                        unreachable!()
                    };
                    self.parse_module_trait_definition(def)?;
                }
            }
        }
        Ok(())
    }

    fn get_function(&self, name: &str) -> Result<FunctionCallSignatures, ValidationError> {
        match self.function_scopes.get(name) {
            None => Err(ValidationError::InvalidFunction(format!(
                "Function {name} does not exist"
            ))),
            Some(t) => Ok(t.clone()),
        }
    }

    fn parse_value(&mut self, value: ObjectValue) {
        self.builder.add_load_instruction(value.into());
    }

    // dont use this for function scopes!
    fn parse_scope(&mut self, scope: Scope, named: &str) -> Result<usize, ValidationError> {
        let current_vars = self.identifiers.clone();
        let current = self.builder.current_scope();
        self.builder.enter_scope(named.to_string(), vec![], None);
        let res = self.builder.current_scope();
        for e in scope.elements {
            self.parse_element(e)?;
        }
        self.builder.exit_scope(current);
        self.identifiers = current_vars;
        Ok(res)
    }

    fn parse_anon_lambda(
        &mut self,
        _fcs: &FunctionCallSignature,
        name: &str,
        fn_args: Vec<FunctionArgument>,
        var_args_start: Option<usize>,
        exp: Expression,
    ) -> Result<(), ValidationError> {
        if var_args_start.is_some() {
            return Err(ValidationError::NotImplemented(format!(
                "var args not implemented for lambdas {name}"
            )));
        }

        let s = match exp {
            Expression::Scope(s) => s,
            e => Scope {
                elements: vec![e.into()],
            },
        };

        let current = self.builder.current_scope();
        let anon = self.builder.enter_scope(
            name.to_string(),
            fn_args
                .iter()
                .map(|a| (a.name.to_string(), false))
                .rev()
                .collect(),
            None,
        );
        let old: Vec<_> = fn_args
            .into_iter()
            .map(|a| {
                (
                    a.name.clone(),
                    self.identifiers.insert(a.name, a.function_type.clone()),
                )
            })
            .collect();
        for exp in s.elements {
            self.parse_element(exp)?;
        }
        old.into_iter().for_each(|(name, rt)| match rt {
            None => {
                self.identifiers.remove(&name);
            }
            Some(s) => {
                self.identifiers.insert(name, s);
            }
        });
        self.builder.exit_scope(current);
        // todo ensure fn_args match signature
        self.builder.add_load_instruction(LoadValue::ScopeId(anon));
        Ok(())
    }
}
