use logos::{Logos, Span};
use rigz_vm::{BinaryOperation, Number, UnaryOperation, VMError, Value};
use std::str::ParseBoolError;

#[derive(Debug, PartialEq, Clone, Default)]
pub enum LexingError {
    NumberParseError,
    #[default]
    NonAsciiError,
    BoolParseError,
    ParseError(String),
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
#[logos(skip r"[ \t\f]+", error = LexingError)]
pub enum TokenKind<'lex> {
    #[token("\n")]
    Newline,
    #[regex("-?[0-9]+", |lex| Value::Number(Number::Int(lex.slice().parse().unwrap())))]
    #[regex("-?[0-9]+\\.[0-9]+", |lex| Value::Number(Number::Float(lex.slice().parse().unwrap())))]
    #[token("none", |_| Value::None)]
    #[token("false", |_| Value::Bool(false))]
    #[token("true", |_| Value::Bool(true))]
    #[regex("('[^'\n\r]+')|(\"[^\"\n\r]+\")|(`[^`\n\r]+`)", |lex| { let s = lex.slice(); Value::String(s[1..s.len()-1].to_string()) })]
    Value(Value<'lex>),
    #[token("=")]
    Assign,
    #[token(";")]
    Semi,
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,
    #[token("as")]
    As,
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
    #[regex("[A-Za-z_]+", |lex| lex.slice())]
    Identifier(&'lex str),
    #[regex("[A-Za-z_$]+", |lex| lex.slice(), priority=3)]
    FunctionIdentifier(&'lex str),
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

#[derive(Debug, Clone)]
pub struct Token<'lex> {
    pub kind: TokenKind<'lex>,
    pub span: Span,
    pub slice: &'lex str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_works() {
        let raw = r#"
            a = 1
            b = 2
            a + b
        "#;

        let lexer = TokenKind::lexer(raw);
        let actual: Vec<TokenKind> = lexer
            .map(|t| t.unwrap())
            .filter(|t| t != &TokenKind::Newline)
            .collect();
        assert_eq!(
            actual,
            vec![
                TokenKind::FunctionIdentifier("a"),
                TokenKind::Assign,
                TokenKind::Value(Value::Number(Number::Int(1))),
                TokenKind::FunctionIdentifier("b"),
                TokenKind::Assign,
                TokenKind::Value(Value::Number(Number::Int(2))),
                TokenKind::FunctionIdentifier("a"),
                TokenKind::BinOp(BinaryOperation::Add),
                TokenKind::FunctionIdentifier("b"),
            ]
        )
    }
}
