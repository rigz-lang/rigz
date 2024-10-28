use rigz_vm::{
    BinaryOperation, Lifecycle, Module, RigzBuilder, RigzType, UnaryOperation, VMBuilder, Value, VM,
};

#[derive(Debug, PartialEq, Clone)]
pub struct Program<'lex> {
    pub elements: Vec<Element<'lex>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature<'vm> {
    pub arguments: Vec<FunctionArgument<'vm>>,
    pub return_type: FunctionType,
    pub self_type: Option<FunctionType>,
    // todo varargs are only valid for positional arguments
    pub positional: bool,
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

impl Into<FunctionType> for RigzType {
    fn into(self) -> FunctionType {
        FunctionType::new(self)
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
    Identifier(&'lex str),
    FilePath(String),
    UrlPath(String),
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
    Import(Exposed<'lex>),
    Export(Exposed<'lex>),
    // todo support later
    // If {
    //     condition: Expression<'lex>,
    //     then: Scope<'lex>,
    //     branch: Option<Scope<'lex>>,
    // },
    // Unless {
    //     condition: Expression<'lex>,
    //     then: Scope<'lex>,
    // },
    // Return(Option<Expression<'lex>>), // import, exports
}

#[derive(Clone, Debug, PartialEq)]
pub enum Assign<'lex> {
    This,
    Identifier(&'lex str, bool),
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
    FunctionCall(&'lex str, Vec<Expression<'lex>>),
    TypeFunctionCall(RigzType, &'lex str, Vec<Expression<'lex>>),
    InstanceFunctionCall(Box<Expression<'lex>>, Vec<&'lex str>, Vec<Expression<'lex>>),
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
    // todo support later
    // Index(Box<Expression<'lex>>, Vec<Expression<'lex>>),
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
