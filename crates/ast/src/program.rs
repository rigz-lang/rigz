use rigz_vm::{BinaryOperation, Lifecycle, RigzType, UnaryOperation, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub input: Option<String>,
    pub elements: Vec<Element>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition {
    pub name: String,
    pub type_definition: FunctionSignature,
    pub body: Scope,
    pub lifecycle: Option<Lifecycle>,
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
    pub default: Option<Value>,
    pub function_type: FunctionType,
    pub var_arg: bool,
    pub rest: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Scope {
    pub elements: Vec<Element>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Statement(Statement),
    Expression(Expression),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImportValue {
    TypeValue(String),
    FilePath(String),
    UrlPath(String),
    // todo support tree shaking?
}

#[derive(Clone, Debug, PartialEq)]
pub enum Exposed {
    TypeValue(String),
    Identifier(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assignment {
        lhs: Assign,
        expression: Expression,
    },
    BinaryAssignment {
        lhs: Assign,
        op: BinaryOperation,
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
}

#[derive(Clone, Debug, PartialEq)]
pub enum Assign {
    This,
    Identifier(String, bool),
    TypedIdentifier(String, bool, RigzType),
    Tuple(Vec<(String, bool)>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RigzArguments {
    Positional(Vec<Expression>),
    Mixed(Vec<Expression>, Vec<(String, Expression)>),
    Named(Vec<(String, Expression)>),
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
    InstanceFunctionCall(Box<Expression>, Vec<String>, RigzArguments),
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
pub enum Expression {
    This,
    Value(Value),
    List(Vec<Expression>),
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
    Unless {
        condition: Box<Expression>,
        then: Scope,
    },
    Error(Box<Expression>),
    Return(Option<Box<Expression>>),
    Index(Box<Expression>, Box<Expression>),
    Tuple(Vec<Expression>),
    Lambda {
        arguments: Vec<FunctionArgument>,
        var_args_start: Option<usize>,
        body: Box<Expression>,
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
    },
    Definition(FunctionDefinition),
}
#[derive(Debug, PartialEq, Clone)]
pub struct TraitDefinition {
    pub name: String,
    pub functions: Vec<FunctionDeclaration>,
}
