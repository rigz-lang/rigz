use crate::prepare::ProgramParser;
use crate::runtime::RuntimeError;
use crate::{FunctionDefinition, FunctionSignature};
use rigz_vm::{BinaryOperation, Module, RigzType, UnaryOperation, Value, VM};

#[derive(Debug, PartialEq, Clone)]
pub struct Program<'lex> {
    pub elements: Vec<Element<'lex>>,
}

impl<'lex> Program<'lex> {
    #[inline]
    /// skips program validation (used for REPL to support statements as last line)
    pub fn create_vm(self) -> Result<VM<'lex>, RuntimeError> {
        let mut builder = ProgramParser::new();
        for element in self.elements {
            builder.parse_element(element).map_err(|e| e.into())?;
        }
        Ok(builder.build())
    }

    #[inline]
    pub fn create_verified_vm(self) -> Result<VM<'lex>, RuntimeError> {
        self.validate().map_err(|e| e.into())?;

        self.create_vm()
    }

    #[inline]
    /// skips program validation (used for REPL to support statements as last line)
    pub fn create_vm_with_modules(
        self,
        modules: Vec<impl Module<'lex> + 'static>,
    ) -> Result<VM<'lex>, RuntimeError> {
        let mut builder = ProgramParser::with_modules(modules).map_err(|e| e.into())?;
        for element in self.elements {
            builder.parse_element(element).map_err(|e| e.into())?;
        }
        Ok(builder.build())
    }

    #[inline]
    pub fn create_verified_vm_with_modules(
        self,
        modules: Vec<impl Module<'lex> + 'static>,
    ) -> Result<VM<'lex>, RuntimeError> {
        self.validate().map_err(|e| e.into())?;
        let mut builder = ProgramParser::with_modules(modules).map_err(|e| e.into())?;
        for element in self.elements {
            builder.parse_element(element).map_err(|e| e.into())?;
        }
        Ok(builder.build())
    }
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
    Index(Box<Expression<'lex>>, Vec<Expression<'lex>>),
}

impl<'lex> From<Vec<Expression<'lex>>> for Expression<'lex> {
    #[inline]
    fn from(value: Vec<Expression<'lex>>) -> Self {
        Expression::List(value)
    }
}

impl<'lex> Expression<'lex> {
    #[inline]
    pub(crate) fn binary(
        lhs: Expression<'lex>,
        op: BinaryOperation,
        rhs: Expression<'lex>,
    ) -> Self {
        Expression::BinExp(Box::new(lhs), op, Box::new(rhs))
    }

    #[inline]
    pub(crate) fn unary(op: UnaryOperation, ex: Expression<'lex>) -> Self {
        Expression::UnaryExp(op, Box::new(ex))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ModuleTraitDefinition<'lex> {
    pub imported: bool,
    pub definition: TraitDefinition<'lex>,
}

impl<'l> ModuleTraitDefinition<'l> {
    pub(crate) fn imported(name: &'l str) -> Self {
        ModuleTraitDefinition {
            imported: true,
            definition: TraitDefinition {
                name,
                functions: vec![],
            },
        }
    }
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
