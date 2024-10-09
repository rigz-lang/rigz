use crate::FunctionDefinition;
use rigz_vm::{BinaryOperation, RigzType, UnaryOperation, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct Program<'lex> {
    pub elements: Vec<Element<'lex>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Element<'lex> {
    Statement(Statement<'lex>),
    Expression(Expression<'lex>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<'lex> {
    Assignment {
        name: &'lex str,
        mutable: bool,
        expression: Expression<'lex>,
    },
    FunctionDefinition {
        name: &'lex str,
        type_definition: FunctionDefinition<'lex>,
        elements: Vec<Element<'lex>>,
    },
    // todo support later
    If {
        condition: Expression<'lex>,
        then: Program<'lex>,
        branch: Option<Program<'lex>>,
    },
    Unless {
        condition: Expression<'lex>,
        then: Program<'lex>,
    },
    Return(Option<Expression<'lex>>), // import, exports
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression<'lex> {
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
    InstanceFunctionCall(Box<Expression<'lex>>, Vec<&'lex str>, Vec<Expression<'lex>>),
    Scope(Vec<Element<'lex>>),
    Cast(Box<Expression<'lex>>, RigzType),
    Symbol(&'lex str),
    If {
        condition: Box<Expression<'lex>>,
        then: Program<'lex>,
        branch: Option<Program<'lex>>,
    },
    Unless {
        condition: Box<Expression<'lex>>,
        then: Program<'lex>,
    },
}

impl <'lex> Expression<'lex> {
    #[inline]
    pub(crate) fn binary(lhs: Expression<'lex>, op: BinaryOperation, rhs: Expression<'lex>) -> Self {
        Expression::BinExp(Box::new(lhs), op, Box::new(rhs))
    }
}
