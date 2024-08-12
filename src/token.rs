use logos::{Logos, Span};
use rigz_vm::{BinaryOperation, Number, UnaryOperation, VMError, Value};
use std::str::ParseBoolError;

#[derive(Debug, PartialEq, Clone, Default)]
pub enum LexingError {
    NumberParseError,
    #[default]
    Other,
    BoolParseError,
}

impl From<std::num::ParseIntError> for LexingError {
    fn from(_: std::num::ParseIntError) -> Self {
        LexingError::NumberParseError
    }
}

impl From<std::num::ParseFloatError> for LexingError {
    fn from(_: std::num::ParseFloatError) -> Self {
        LexingError::NumberParseError
    }
}

impl From<ParseBoolError> for LexingError {
    fn from(_: ParseBoolError) -> Self {
        LexingError::BoolParseError
    }
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(error = LexingError)]
pub enum TokenKind {
    #[regex(r"[ \t\f]+", logos::skip)]
    Ignored,
    #[token("\n")]
    Newline,
    #[regex("-?[0-9]+", |lex| lex.slice().parse())]
    Integer(i64),
    #[regex("-?[0-9]+\\.[0-9]+", |lex| lex.slice().parse())]
    Float(f64),
    #[regex("true|false", |lex| lex.slice().parse())]
    Bool(bool),
    #[regex("('[^'\n\r]+')|(\"[^\"\n\r]+\")|(`[^`\n\r]+`)", |lex| { let s = lex.slice(); s[1..s.len()-1].to_string() })]
    StrLiteral(String),
    // TODO double & backticks string support interpolation
    #[token("none")]
    None,
    #[token("=")]
    Assign,
    #[token("==", |_| BinaryOperation::Eq)]
    #[token("!=", |_| BinaryOperation::Neq)]
    #[token("<", |_| BinaryOperation::Lt)]
    #[token(">>", |_| BinaryOperation::Shr)]
    #[token("<<", |_| BinaryOperation::Shl)]
    #[token(">", |_| BinaryOperation::Gt)]
    #[token("<=", |_| BinaryOperation::Lte)]
    #[token(">=", |_| BinaryOperation::Gte)]
    #[token("+", |_| BinaryOperation::Add)]
    #[token("*", |_| BinaryOperation::Mul)]
    #[token("/", |_| BinaryOperation::Div)]
    #[token("%", |_| BinaryOperation::Rem)]
    #[token("&&", |_| BinaryOperation::And)]
    #[token("||", |_| BinaryOperation::Or)]
    #[token("&", |_| BinaryOperation::BitAnd)]
    #[token("|", |_| BinaryOperation::BitOr)]
    #[token("^", |_| BinaryOperation::Xor)]
    BinOp(BinaryOperation),
    #[token("!")]
    Not,
    #[token("-")]
    Minus,
    #[token(".")]
    Period,
    #[token(",")]
    Comma,
    #[token("fn")]
    FunctionDef,
    #[regex("[A-Za-z_]+", |lex| lex.slice().to_string())]
    Identifier(String),
    #[regex("[A-Za-z_$]+", |lex| lex.slice().to_string(), priority=3)]
    FunctionIdentifier(String),
    #[token("(")]
    Lparen,
    #[token(")")]
    Rparen,
    #[token("{")]
    Lcurly,
    #[token("}")]
    Rcurly,
    #[token("[")]
    Lbracket,
    #[token("]")]
    Rbracket,
    #[token("do")]
    Do,
    #[token("end")]
    End,
}

impl TokenKind {
    #[inline]
    pub fn is_value(&self) -> bool {
        match self {
            TokenKind::None
            | TokenKind::Bool(_)
            | TokenKind::Integer(_)
            | TokenKind::Float(_)
            | TokenKind::StrLiteral(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_identifier(&self) -> bool {
        match &self {
            TokenKind::Identifier(_) => true,
            TokenKind::FunctionIdentifier(_) => true,
            _ => false
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token<'vm> {
    pub(crate) kind: TokenKind,
    pub(crate) span: Span,
    pub(crate) slice: &'vm str,
}

impl<'vm> Token<'vm> {
    #[inline]
    pub fn parse_error(&self, message: String) -> VMError {
        VMError::ParseError(message, self.span.start, self.span.end)
    }

    #[inline]
    pub fn is_identifier(&self) -> bool {
        self.kind.is_identifier()
    }

    #[inline]
    pub fn is_value(&self) -> bool {
        self.kind.is_value()
    }

    pub fn to_value(&self) -> Option<Value<'vm>> {
        let v = match &self.kind {
            TokenKind::Integer(i) => Value::Number(Number::Int(*i)),
            TokenKind::Float(f) => Value::Number(Number::Float(*f)),
            TokenKind::Bool(b) => Value::Bool(*b),
            TokenKind::StrLiteral(s) => Value::String(s.to_string()),
            TokenKind::None => Value::None,
            _ => return None,
        };
        Some(v)
    }
}

impl<'vm> Default for Token<'vm> {
    fn default() -> Self {
        Self {
            kind: TokenKind::Ignored,
            slice: "",
            span: Span::default(),
        }
    }
}
