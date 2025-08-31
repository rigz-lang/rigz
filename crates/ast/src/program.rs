use crate::ParseError;
use itertools::Itertools;
use rigz_core::{BinaryAssignOperation, BinaryOperation, EnumDeclaration, Lifecycle, PrimitiveValue, RigzType, UnaryOperation};
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Program {
    pub input: String,
    pub elements: Vec<Element>,
    pub errors: Vec<ParseError>,
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.elements.iter().map(|e| e.to_string()).join("\n")
        )
    }
}

impl Program {
    pub fn for_elements(elements: Vec<Element>) -> Self {
        Self {
            elements,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArgType {
    Positional,
    List,
    Map,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    pub arguments: Vec<FunctionArgument>,
    pub return_type: FunctionType,
    pub self_type: Option<FunctionType>,
    pub var_args_start: Option<usize>,
    pub arg_type: ArgType,
}

impl Display for FunctionSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition {
    pub name: String,
    pub type_definition: FunctionSignature,
    pub body: Scope,
    pub lifecycle: Option<Lifecycle>,
    pub docs: Option<String>,
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub rigz_type: RigzType,
    pub mutable: bool,
}

impl From<RigzType> for FunctionType {
    fn from(val: RigzType) -> Self {
        FunctionType::new(val)
    }
}

impl FunctionType {
    pub fn new(rigz_type: RigzType) -> Self {
        Self {
            rigz_type,
            mutable: false,
        }
    }

    pub fn mutable(rigz_type: RigzType) -> Self {
        Self {
            rigz_type,
            mutable: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionArgument {
    pub name: String,
    pub default: Option<Expression>,
    pub function_type: FunctionType,
    pub var_arg: bool,
    pub rest: bool,
}

impl Display for FunctionArgument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.function_type.rigz_type)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Scope {
    pub elements: Vec<Element>,
}

impl Display for Scope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.elements.iter().map(|e| e.to_string()).join("\n")
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Statement(Statement),
    Expression(Expression),
}

impl Display for Element {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Element::Statement(s) => s.to_string(),
            Element::Expression(e) => e.to_string(),
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImportValue {
    TypeValue(String),
    FilePath(String),
    UrlPath(String),
    // todo support tree shaking?
}

impl Display for ImportValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ImportValue::TypeValue(s) => s,
            ImportValue::FilePath(s) => s,
            ImportValue::UrlPath(s) => s,
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Exposed {
    TypeValue(String),
    Identifier(String),
}

impl Display for Exposed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Exposed::TypeValue(s) => s,
            Exposed::Identifier(s) => s,
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assignment {
        lhs: Assign,
        expression: Expression,
    },
    BinaryAssignment {
        lhs: Assign,
        op: BinaryAssignOperation,
        expression: Expression,
    },
    FunctionDefinition(FunctionDefinition),
    Trait(TraitDefinition),
    Import(ImportValue),
    Export(Exposed),
    TypeDefinition(String, RigzType),
    TraitImpl {
        base_trait: RigzType,
        concrete: RigzType,
        definitions: Vec<FunctionDefinition>,
    },
    ObjectDefinition(ObjectDefinition),
    Enum(EnumDeclaration),
    For {
        each: Each,
        expression: Expression,
        body: Scope,
    },
    Loop(Scope),
    Module(String, Vec<Element>)
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Assignment { lhs, expression } => write!(f, "{lhs} = {expression}"),
            Statement::BinaryAssignment {
                lhs,
                op,
                expression,
            } => write!(f, "{lhs} {op} {expression}"),
            Statement::FunctionDefinition(func) => write!(f, "{func}"),
            Statement::Trait(t) => write!(f, "{t}"),
            Statement::Import(s) => write!(f, "import {s}"),
            Statement::Export(e) => write!(f, "export {e}"),
            Statement::TypeDefinition(t, def) => write!(f, "type {t} = {def}"),
            Statement::TraitImpl {
                base_trait,
                concrete,
                definitions,
            } => write!(
                f,
                "impl {base_trait} for {concrete}\n{}\nend",
                definitions.iter().map(|d| d.to_string()).join("\n")
            ),
            Statement::ObjectDefinition(obj) => write!(f, "{obj}"),
            Statement::Enum(en) => write!(f, "{en}"),
            Statement::For {
                each,
                expression,
                body,
            } => write!(f, "for {each} in {expression} do \n{body}\nend"),
            Statement::Loop(body) => write!(f, "loop\n{body}\nend"),
            Statement::Module(rt, elements) => write!(f, "mod {rt}\n{}\nend", elements.iter().map(|e| e.to_string()).join("\n")),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AssignIndex {
    Identifier(String),
    Index(Expression),
}

impl Display for AssignIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignIndex::Identifier(id) => write!(f, "{id}"),
            AssignIndex::Index(idx) => write!(f, "[{idx}]"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Assign {
    This,
    Identifier {
        name: String,
        mutable: bool,
        shadow: bool,
    },
    TypedIdentifier {
        name: String,
        mutable: bool,
        shadow: bool,
        rigz_type: RigzType,
    },
    Tuple(Vec<(String, bool, bool)>),
    InstanceSet(Expression, Vec<AssignIndex>),
}

impl Display for Assign {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Assign::This => write!(f, "self"),
            Assign::Identifier { name, mutable, .. } if *mutable => write!(f, "mut {name}"),
            Assign::Identifier { name, shadow, .. } if *shadow => write!(f, "let {name}"),
            Assign::Identifier { name, .. } => write!(f, "{name}"),
            Assign::TypedIdentifier {
                name,
                mutable,
                rigz_type,
                ..
            } if *mutable => write!(f, "mut {name}: {rigz_type}"),
            Assign::TypedIdentifier {
                name,
                shadow,
                rigz_type,
                ..
            } if *shadow => write!(f, "let {name}: {rigz_type}"),
            Assign::TypedIdentifier {
                name, rigz_type, ..
            } => write!(f, "{name}: {rigz_type}"),
            Assign::Tuple(v) => write!(
                f,
                "({})",
                v.iter()
                    .map(|v| {
                        if v.1 {
                            format!("mut {}", v.0)
                        } else if v.2 {
                            format!("let {}", v.0)
                        } else {
                            v.0.to_string()
                        }
                    })
                    .join(", ")
            ),
            Assign::InstanceSet(e, index) => {
                write!(f, "{e}{}", index.iter().map(|v| v.to_string()).join("."))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RigzArguments {
    Positional(Vec<Expression>),
    Mixed(Vec<Expression>, Vec<(String, Expression)>),
    Named(Vec<(String, Expression)>),
}

impl Display for RigzArguments {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RigzArguments::Positional(p) => {
                if !p.is_empty() {
                    write!(f, " {}", p.iter().map(|f| f.to_string()).join(", "))
                } else {
                    write!(f, "")
                }
            }
            RigzArguments::Mixed(m, n) => write!(
                f,
                " {}, {}",
                m.iter().map(|f| f.to_string()).join(", "),
                n.iter().map(|(k, v)| format!("{k}: {v}")).join(", ")
            ),
            RigzArguments::Named(n) => write!(
                f,
                " {}",
                n.iter().map(|(k, v)| format!("{k}: {v}")).join(", ")
            ),
        }
    }
}

impl RigzArguments {
    pub fn prepend(self, base: Expression) -> Self {
        match self {
            RigzArguments::Positional(a) => {
                let mut p = Vec::with_capacity(a.len() + 1);
                p.push(base);
                p.extend(a);
                RigzArguments::Positional(p)
            }
            RigzArguments::Mixed(a, m) => {
                let mut p = Vec::with_capacity(a.len() + 1);
                p.push(base);
                p.extend(a);
                RigzArguments::Mixed(p, m)
            }
            RigzArguments::Named(n) => RigzArguments::Mixed(vec![base], n),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            RigzArguments::Positional(s) => s.len(),
            RigzArguments::Mixed(a, b) => a.len() + b.len(),
            RigzArguments::Named(n) => n.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            RigzArguments::Positional(s) => s.is_empty(),
            RigzArguments::Mixed(a, b) => a.is_empty() && b.is_empty(),
            RigzArguments::Named(n) => n.is_empty(),
        }
    }
}

impl From<Vec<Expression>> for RigzArguments {
    fn from(value: Vec<Expression>) -> Self {
        RigzArguments::Positional(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionExpression {
    FunctionCall(String, RigzArguments),
    TypeFunctionCall(RigzType, String, RigzArguments),
    TypeConstructor(RigzType, RigzArguments),
    InstanceFunctionCall(Box<Expression>, Vec<String>, RigzArguments),
}

impl Display for FunctionExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionExpression::FunctionCall(n, a) => write!(f, "{n}{a}"),
            FunctionExpression::TypeFunctionCall(t, n, a) => write!(f, "{t}.{n}{a}"),
            FunctionExpression::TypeConstructor(n, a) => write!(f, "{n}{a}"),
            FunctionExpression::InstanceFunctionCall(ex, n, a) => {
                write!(f, "{ex}.{}{a}", n.join("."))
            }
        }
    }
}

impl FunctionExpression {
    pub fn prepend(self, expression: Expression) -> Self {
        match self {
            FunctionExpression::FunctionCall(n, args) => {
                FunctionExpression::FunctionCall(n, args.prepend(expression))
            }
            FunctionExpression::TypeFunctionCall(t, name, args) => {
                FunctionExpression::TypeFunctionCall(t, name, args.prepend(expression))
            }
            FunctionExpression::TypeConstructor(t, args) => {
                FunctionExpression::TypeConstructor(t, args.prepend(expression))
            }
            FunctionExpression::InstanceFunctionCall(n, calls, args) => {
                FunctionExpression::InstanceFunctionCall(n, calls, args.prepend(expression))
            }
        }
    }
}

impl From<FunctionExpression> for Expression {
    fn from(value: FunctionExpression) -> Self {
        Expression::Function(value)
    }
}

impl From<FunctionExpression> for Box<Expression> {
    fn from(value: FunctionExpression) -> Self {
        Box::new(value.into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MatchVariant {
    Enum {
        name: String,
        condition: MatchVariantCondition,
        body: Scope,
        variables: Vec<MatchVariantVariable>,
    },
    Else(Scope),
}

impl Display for MatchVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MatchVariantVariable {
    Identifier(String),
    Value(PrimitiveValue),
}

#[derive(Clone, Debug, PartialEq)]
pub enum MatchVariantCondition {
    None,
    If(Expression),
    Unless(Expression),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    This,
    Value(PrimitiveValue),
    List(Vec<Expression>),
    Set(Vec<Expression>),
    Map(Vec<(Expression, Expression)>),
    Identifier(String),
    BinExp(Box<Expression>, BinaryOperation, Box<Expression>),
    UnaryExp(UnaryOperation, Box<Expression>),
    Function(FunctionExpression),
    Scope(Scope),
    Cast(Box<Expression>, RigzType),
    Symbol(String),
    If {
        condition: Box<Expression>,
        then: Scope,
        branch: Option<Scope>,
    },
    Ternary {
        condition: Box<Expression>,
        then: Box<Expression>,
        branch: Box<Expression>,
    },
    Unless {
        condition: Box<Expression>,
        then: Scope,
    },
    IfGuard {
        condition: Box<Expression>,
        then: Box<Expression>,
    },
    UnlessGuard {
        condition: Box<Expression>,
        then: Box<Expression>,
    },
    Enum(String, String, Option<Box<Expression>>),
    Match {
        condition: Box<Expression>,
        variants: Vec<MatchVariant>,
    },
    Error(Box<Expression>),
    Return(Option<Box<Expression>>),
    Exit(Option<Box<Expression>>),
    Index(Box<Expression>, Box<Expression>),
    Tuple(Vec<Expression>),
    Lambda {
        arguments: Vec<FunctionArgument>,
        var_args_start: Option<usize>,
        body: Box<Element>,
    },
    ForList {
        var: String,
        expression: Box<Expression>,
        body: Box<Expression>,
    },
    ForMap {
        k_var: String,
        v_var: String,
        expression: Box<Expression>,
        key: Box<Expression>,
        value: Option<Box<Expression>>,
    },
    Into {
        base: Box<Expression>,
        next: FunctionExpression,
    },
    DoubleBang(Box<Expression>),
    Try(Box<Expression>),
    Catch {
        base: Box<Expression>,
        var: Option<String>,
        catch: Scope,
    },
    Break,
    Next,
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::This => write!(f, "self"),
            Expression::Value(PrimitiveValue::String(s)) => write!(f, "'{s}'"),
            Expression::Value(v) => write!(f, "{v}"),
            Expression::List(v) => write!(f, "[{}]", v.iter().map(|v| v.to_string()).join(", ")),
            Expression::Set(v) => write!(f, "Set[{}]", v.iter().map(|v| v.to_string()).join(", ")),
            Expression::Map(m) => write!(
                f,
                "{{{}}}",
                m.iter().map(|(k, v)| format!("{k} = {v}")).join(", ")
            ),
            Expression::Identifier(id) => write!(f, "{id}"),
            Expression::BinExp(lhs, op, rhs) => write!(f, "({lhs} {op} {rhs})"),
            Expression::UnaryExp(op, exp) => write!(f, "{op}{exp}"),
            Expression::Function(func) => write!(f, "{func}"),
            Expression::Scope(s) => write!(f, "do\n{s}\nend"),
            Expression::Cast(ex, rt) => write!(f, "{ex} as {rt}"),
            Expression::Symbol(s) => write!(f, ":{s}"),
            Expression::If {
                condition,
                then,
                branch,
            } => match branch {
                None => write!(f, "if {condition}\n{then}\nend"),
                Some(branch) => write!(f, "if {condition}\n{then}\nelse\n{branch}\nend"),
            },
            Expression::Ternary {
                condition,
                then,
                branch,
            } => write!(f, "{condition} ? {then} : {branch}"),
            Expression::Unless { condition, then } => write!(f, "unless {condition}\n{then}\nend"),
            Expression::IfGuard { condition, then } => write!(f, "{condition} if {then}"),
            Expression::UnlessGuard { condition, then } => write!(f, "{condition} unless {then}"),
            Expression::Enum(e, v, None) => write!(f, "{e}.{v}"),
            Expression::Enum(e, v, Some(exp)) => write!(f, "{e}.{v}({exp})"),
            Expression::Match {
                condition,
                variants,
            } => write!(
                f,
                "match {condition}\n{}\nend",
                variants.iter().map(|v| v.to_string()).join("\n")
            ),
            Expression::Error(e) => write!(f, "raise {e}"),
            Expression::Return(None) => write!(f, "return"),
            Expression::Exit(None) => write!(f, "exit"),
            Expression::Return(Some(e)) => write!(f, "return {e}"),
            Expression::Exit(Some(e)) => write!(f, "exit {e}"),
            Expression::Index(b, i) => write!(f, "{b}[{i}]"),
            Expression::Tuple(t) => write!(f, "({})", t.iter().map(|v| v.to_string()).join(", ")),
            Expression::Lambda {
                arguments,
                var_args_start,
                body,
            } => write!(f, "{{|{}| {body}}}", fn_args(arguments, var_args_start)),
            Expression::ForList {
                var,
                expression,
                body,
            } => write!(f, "[for {var} in {expression}: {body}]"),
            Expression::ForMap {
                k_var,
                v_var,
                expression,
                key,
                value,
            } => {
                let v = match value {
                    None => key.to_string(),
                    Some(v) => format!("{key}, {v}"),
                };
                write!(f, "{{for {k_var}, {v_var} in {expression}: {v}}}")
            }
            Expression::Into { base, next } => write!(f, "{base} |> {next}"),
            Expression::DoubleBang(ex) => write!(f, "{ex}!!"),
            Expression::Try(exp) => write!(f, "try {exp}"),
            Expression::Catch { base, var, catch } => match var {
                None => write!(f, "{base} catch\n{catch}\nend"),
                Some(v) => write!(f, "{base} catch |{v}|\n{catch}\nend"),
            },
            Expression::Break => write!(f, "break"),
            Expression::Next => write!(f, "next"),
        }
    }
}

fn fn_args(arguments: &[FunctionArgument], var_args_start: &Option<usize>) -> String {
    arguments
        .iter()
        .enumerate()
        .map(|(idx, arg)| {
            let f = match var_args_start {
                &Some(i) if idx == i => "var ",
                _ => "",
            };
            format!("{f}{arg}")
        })
        .join(", ")
}

#[derive(Clone, Debug, PartialEq)]
pub enum Each {
    Identifier {
        name: String,
        mutable: bool,
        shadow: bool,
    },
    TypedIdentifier {
        name: String,
        mutable: bool,
        shadow: bool,
        rigz_type: RigzType,
    },
    Tuple(Vec<(String, bool, bool)>),
}

impl Display for Each {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Each::Identifier { name, mutable, .. } if *mutable => write!(f, "mut {name}"),
            Each::Identifier { name, shadow, .. } if *shadow => write!(f, "let {name}"),
            Each::Identifier { name, .. } => write!(f, "{name}"),
            Each::TypedIdentifier {
                name,
                mutable,
                rigz_type,
                ..
            } if *mutable => write!(f, "mut {name}: {rigz_type}"),
            Each::TypedIdentifier {
                name,
                shadow,
                rigz_type,
                ..
            } if *shadow => write!(f, "let {name}: {rigz_type}"),
            Each::TypedIdentifier {
                name, rigz_type, ..
            } => write!(f, "{name}: {rigz_type}"),
            Each::Tuple(v) => write!(
                f,
                "({})",
                v.iter()
                    .map(|v| {
                        if v.1 {
                            format!("mut {}", v.0)
                        } else if v.2 {
                            format!("let {}", v.0)
                        } else {
                            v.0.to_string()
                        }
                    })
                    .join(", ")
            ),
        }
    }
}

impl From<Vec<Expression>> for Expression {
    #[inline]
    fn from(value: Vec<Expression>) -> Self {
        Expression::List(value)
    }
}

impl Expression {
    #[inline]
    pub fn binary(lhs: Expression, op: BinaryOperation, rhs: Expression) -> Self {
        Expression::BinExp(Box::new(lhs), op, Box::new(rhs))
    }

    #[inline]
    pub(crate) fn unary(op: UnaryOperation, ex: Expression) -> Self {
        Expression::UnaryExp(op, Box::new(ex))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ModuleTraitDefinition {
    pub auto_import: bool,
    pub definition: TraitDefinition,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FunctionDeclaration {
    Declaration {
        name: String,
        type_definition: FunctionSignature,
        docs: Option<String>,
    },
    Definition(FunctionDefinition),
}

impl Display for FunctionDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionDeclaration::Declaration {
                name,
                type_definition,
                docs: _,
            } => write!(f, "fn {name}{type_definition}"),
            FunctionDeclaration::Definition(def) => write!(f, "{def}"),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct TraitDefinition {
    pub name: String,
    pub functions: Vec<FunctionDeclaration>,
}

impl Display for TraitDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "trait {}\n{}\nend",
            self.name,
            self.functions.iter().map(|f| f.to_string()).join(", ")
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ObjectAttr {
    pub name: String,
    pub attr_type: FunctionType,
    pub default: Option<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ObjectDefinition {
    pub rigz_type: RigzType,
    pub fields: Vec<ObjectAttr>,
    pub constructors: Vec<Constructor>,
    pub functions: Vec<FunctionDeclaration>,
}

impl Display for ObjectDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Constructor {
    Default,
    Declaration(Vec<FunctionArgument>, Option<usize>),
    Definition(Vec<FunctionArgument>, Option<usize>, Scope),
}
