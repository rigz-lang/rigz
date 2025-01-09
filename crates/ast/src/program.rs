use rigz_vm::{BinaryOperation, Lifecycle, RigzType, UnaryOperation, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct Program<'lex> {
    pub elements: Vec<Element<'lex>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArgType {
    Positional,
    List,
    Map,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature<'vm> {
    pub arguments: Vec<FunctionArgument<'vm>>,
    pub return_type: FunctionType,
    pub self_type: Option<FunctionType>,
    pub var_args_start: Option<usize>,
    pub arg_type: ArgType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition<'lex> {
    pub name: &'lex str,
    pub type_definition: FunctionSignature<'lex>,
    pub body: Scope<'lex>,
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
pub struct FunctionArgument<'vm> {
    pub name: &'vm str,
    pub default: Option<Value>,
    pub function_type: FunctionType,
    pub var_arg: bool,
    pub rest: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Scope<'lex> {
    pub elements: Vec<Element<'lex>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Element<'lex> {
    Statement(Statement<'lex>),
    Expression(Expression<'lex>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImportValue<'lex> {
    TypeValue(&'lex str),
    FilePath(&'lex str),
    UrlPath(&'lex str),
    // todo support tree shaking?
}

#[derive(Clone, Debug, PartialEq)]
pub enum Exposed<'lex> {
    TypeValue(&'lex str),
    Identifier(&'lex str),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<'lex> {
    Assignment {
        lhs: Assign<'lex>,
        expression: Expression<'lex>,
    },
    BinaryAssignment {
        lhs: Assign<'lex>,
        op: BinaryOperation,
        expression: Expression<'lex>,
    },
    FunctionDefinition(FunctionDefinition<'lex>),
    Trait(TraitDefinition<'lex>),
    Import(ImportValue<'lex>),
    Export(Exposed<'lex>),
    TypeDefinition(&'lex str, RigzType),
    TraitImpl {
        base_trait: RigzType,
        concrete: RigzType,
        definitions: Vec<FunctionDefinition<'lex>>,
    }, // todo support later
       // If {
       //     condition: Expression<'lex>,
       //     then: Scope<'lex>,
       //     branch: Option<Scope<'lex>>,
       // },
       // Unless {
       //     condition: Expression<'lex>,
       //     then: Scope<'lex>,
       // },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Assign<'lex> {
    This,
    Identifier(&'lex str, bool),
    TypedIdentifier(&'lex str, bool, RigzType),
    Tuple(Vec<(&'lex str, bool)>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RigzArguments<'lex> {
    Positional(Vec<Expression<'lex>>),
    Mixed(Vec<Expression<'lex>>, Vec<(&'lex str, Expression<'lex>)>),
    Named(Vec<(&'lex str, Expression<'lex>)>),
}

impl<'a> RigzArguments<'a> {
    pub fn prepend(self, base: Expression<'a>) -> Self {
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

impl<'lex> From<Vec<Expression<'lex>>> for RigzArguments<'lex> {
    fn from(value: Vec<Expression<'lex>>) -> Self {
        RigzArguments::Positional(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionExpression<'lex> {
    FunctionCall(&'lex str, RigzArguments<'lex>),
    TypeFunctionCall(RigzType, &'lex str, RigzArguments<'lex>),
    InstanceFunctionCall(Box<Expression<'lex>>, Vec<&'lex str>, RigzArguments<'lex>),
}

impl<'lex> FunctionExpression<'lex> {
    pub(crate) fn prepend(self, expression: Expression<'lex>) -> Self {
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

impl<'lex> From<FunctionExpression<'lex>> for Expression<'lex> {
    fn from(value: FunctionExpression<'lex>) -> Self {
        Expression::Function(value)
    }
}

impl<'lex> From<FunctionExpression<'lex>> for Box<Expression<'lex>> {
    fn from(value: FunctionExpression<'lex>) -> Self {
        Box::new(value.into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression<'lex> {
    This,
    Value(Value),
    List(Vec<Expression<'lex>>),
    Map(Vec<(Expression<'lex>, Expression<'lex>)>),
    Identifier(&'lex str),
    BinExp(
        Box<Expression<'lex>>,
        BinaryOperation,
        Box<Expression<'lex>>,
    ),
    UnaryExp(UnaryOperation, Box<Expression<'lex>>),
    Function(FunctionExpression<'lex>),
    Scope(Scope<'lex>),
    Cast(Box<Expression<'lex>>, RigzType),
    Symbol(&'lex str),
    If {
        condition: Box<Expression<'lex>>,
        then: Scope<'lex>,
        branch: Option<Scope<'lex>>,
    },
    Unless {
        condition: Box<Expression<'lex>>,
        then: Scope<'lex>,
    },
    Error(Box<Expression<'lex>>),
    Return(Option<Box<Expression<'lex>>>),
    Index(Box<Expression<'lex>>, Box<Expression<'lex>>),
    Tuple(Vec<Expression<'lex>>),
    Lambda {
        arguments: Vec<FunctionArgument<'lex>>,
        var_args_start: Option<usize>,
        body: Box<Expression<'lex>>,
    },
    ForList {
        var: &'lex str,
        expression: Box<Expression<'lex>>,
        body: Box<Expression<'lex>>,
    },
    ForMap {
        k_var: &'lex str,
        v_var: &'lex str,
        expression: Box<Expression<'lex>>,
        key: Box<Expression<'lex>>,
        value: Option<Box<Expression<'lex>>>,
    },
    Into {
        base: Box<Expression<'lex>>,
        next: FunctionExpression<'lex>,
    },
}

impl<'lex> From<Vec<Expression<'lex>>> for Expression<'lex> {
    #[inline]
    fn from(value: Vec<Expression<'lex>>) -> Self {
        Expression::List(value)
    }
}

impl<'lex> Expression<'lex> {
    #[inline]
    pub fn binary(lhs: Expression<'lex>, op: BinaryOperation, rhs: Expression<'lex>) -> Self {
        Expression::BinExp(Box::new(lhs), op, Box::new(rhs))
    }

    #[inline]
    pub(crate) fn unary(op: UnaryOperation, ex: Expression<'lex>) -> Self {
        Expression::UnaryExp(op, Box::new(ex))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ModuleTraitDefinition<'lex> {
    pub auto_import: bool,
    pub definition: TraitDefinition<'lex>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FunctionDeclaration<'lex> {
    Declaration {
        name: &'lex str,
        type_definition: FunctionSignature<'lex>,
    },
    Definition(FunctionDefinition<'lex>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct TraitDefinition<'lex> {
    pub name: &'lex str,
    pub functions: Vec<FunctionDeclaration<'lex>>,
}
