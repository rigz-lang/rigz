use logos::{Logos, Span};
use rigz_vm::{BinaryOperation, Number, Value};
use std::fmt::{Debug, Display, Formatter};
use std::str::ParseBoolError;

#[derive(Debug, PartialEq, Clone, Default)]
pub enum ParsingError {
    NumberParseError,
    #[default]
    NonAsciiError,
    BoolParseError,
    ParseError(String),
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::NumberParseError => write!(f, "Invalid Number"),
            ParsingError::NonAsciiError => write!(f, "Invalid Character"),
            ParsingError::BoolParseError => write!(f, "Invalid Bool"),
            ParsingError::ParseError(s) => write!(f, "{}", s),
        }
    }
}

impl From<std::num::ParseIntError> for ParsingError {
    fn from(_: std::num::ParseIntError) -> Self {
        ParsingError::NumberParseError
    }
}

impl From<std::num::ParseFloatError> for ParsingError {
    fn from(_: std::num::ParseFloatError) -> Self {
        ParsingError::NumberParseError
    }
}

impl From<ParseBoolError> for ParsingError {
    fn from(_: ParseBoolError) -> Self {
        ParsingError::BoolParseError
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) struct Symbol<'lex>(pub(crate) &'lex str);

impl Display for Symbol<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, ":{}", self.0)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub(crate) enum TokenValue<'lex> {
    #[default]
    None,
    Bool(bool),
    Number(Number),
    String(&'lex str),
}

impl Display for TokenValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenValue::None => write!(f, "none"),
            TokenValue::Bool(v) => write!(f, "{}", v),
            TokenValue::Number(v) => write!(f, "{}", v),
            TokenValue::String(v) => write!(f, "{}", v),
        }
    }
}

impl Into<Value> for TokenValue<'_> {
    fn into(self) -> Value {
        match self {
            TokenValue::None => Value::None,
            TokenValue::Bool(b) => Value::Bool(b),
            TokenValue::Number(n) => Value::Number(n),
            TokenValue::String(s) => Value::String(s.to_string()),
        }
    }
}

#[derive(Logos, Copy, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\f]+", error = ParsingError)]
pub(crate) enum TokenKind<'lex> {
    #[token("\n")]
    Newline,
    #[token("none", |_| TokenValue::None)]
    #[token("false", |_| TokenValue::Bool(false))]
    #[token("true", |_| TokenValue::Bool(true))]
    #[regex("-?[0-9][0-9_]*\\.[0-9][0-9_]*", |lex| TokenValue::Number(lex.slice().parse().unwrap()))]
    #[regex("-?[0-9][0-9_]*", |lex| TokenValue::Number(lex.slice().parse().unwrap()))]
    // todo special logic to support string escape expressions, probably as dedicated tokens
    #[regex("('[^'\n\r]*')|(\"[^\"\n\r]*\")|(`[^`\n\r]*`)", |lex| { let s = lex.slice(); TokenValue::String(&s[1..s.len()-1]) })]
    Value(TokenValue<'lex>),
    #[token("=")]
    Assign,
    #[token(";")]
    Semi,
    #[token(":")]
    Colon,
    #[token("->")]
    Arrow,
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
    #[token("^", |_| BinaryOperation::Xor)]
    #[token("?:", |_| BinaryOperation::Elvis)]
    BinOp(BinaryOperation),
    #[token(">>=", |_| BinaryOperation::Shr)]
    #[token("<<=", |_| BinaryOperation::Shl)]
    #[token("+=", |_| BinaryOperation::Add)]
    #[token("-=", |_| BinaryOperation::Sub)]
    #[token("*=", |_| BinaryOperation::Mul)]
    #[token("/=", |_| BinaryOperation::Div)]
    #[token("%=", |_| BinaryOperation::Rem)]
    #[token("&&=", |_| BinaryOperation::And)]
    #[token("||=", |_| BinaryOperation::Or)]
    #[token("&=", |_| BinaryOperation::BitAnd)]
    #[token("|=", |_| BinaryOperation::BitOr)]
    #[token("^=", |_| BinaryOperation::Xor)]
    BinAssign(BinaryOperation),
    #[token("!")]
    Not,
    #[regex("[A-Z][A-Za-z0-9_]+!?\\??", |lex| lex.slice())]
    TypeValue(&'lex str),
    #[token("-")]
    Minus,
    #[token("|")]
    Pipe,
    #[token("&")]
    And,
    #[token(".")]
    Period,
    #[token(",")]
    Comma,
    #[token("fn")]
    FunctionDef,
    #[regex("\\$[a-z_]?[A-Za-z0-9_]*", |lex| lex.slice())]
    #[regex("[a-z_][A-Za-z0-9_]*", |lex| lex.slice())]
    Identifier(&'lex str),
    #[regex(":[A-Za-z0-9_]+", |lex| { let s = lex.slice(); Symbol(&s[1..]) })]
    Symbol(Symbol<'lex>),
    #[regex("@[a-z_][A-Za-z0-9_]*", |lex| { let s = lex.slice(); &s[1..] })]
    Lifecycle(&'lex str),
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
    #[token("if")]
    If,
    #[token("unless")]
    Unless,
    #[token("else")]
    Else,
    #[token("type")]
    Type,
    #[token("trait")]
    Trait,
    #[token("++")]
    Increment,
    #[token("--")]
    Decrement,
    #[token("self")]
    This,
    #[regex("#[^\n]*")]
    #[regex("/\\*(?:[^*]|\\*[^/])*\\*/")]
    Comment, //todo support doc-tests, nested comments
    // Reserved for future versions
    #[regex("\\$[0-9]+", |lex| { let s = lex.slice(); s[1..].parse::<usize>().unwrap() })]
    Arg(usize),
    #[token("return")]
    Return,
    #[token("import")]
    Import,
    #[token("export")]
    Export,
    #[token("var")]
    VariableArgs,
    #[token("mod")]
    Module,
    #[token("raise")]
    Error,
    #[token("=>")]
    Into,
    #[token("..")]
    Range,
    #[token("..=")]
    RangeInclusive,
    #[token("?")]
    Optional,
    #[token("!!")]
    DoubleBang,
    // todo support zig style try / catch
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    // todo should puts & log be dedicated tokens to since they're VM instructions?
    // #[regex("[A-Z][a-z_]+\\??!?", |lex| lex.slice())]
    // TypeValue(&'lex str),
}

impl Display for TokenKind<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Newline => write!(f, "\\n"),
            TokenKind::Value(v) => write!(f, "{}", v),
            TokenKind::Assign => write!(f, "="),
            TokenKind::Semi => write!(f, ";"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Into => write!(f, "=>"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::As => write!(f, "as"),
            TokenKind::BinOp(op) => write!(f, "{}", op),
            TokenKind::BinAssign(op) => write!(f, "{}=", op),
            TokenKind::Not => write!(f, "!"),
            TokenKind::TypeValue(t) => write!(f, "{}", *t),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::And => write!(f, "&"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Period => write!(f, "."),
            TokenKind::Comma => write!(f, ","),
            TokenKind::FunctionDef => write!(f, "fn"),
            TokenKind::Identifier(id) => write!(f, "{}", id),
            TokenKind::Symbol(s) => write!(f, "{}", s),
            TokenKind::Lifecycle(s) => write!(f, "@{}", s),
            TokenKind::Lparen => write!(f, "("),
            TokenKind::Rparen => write!(f, ")"),
            TokenKind::Lcurly => write!(f, "{{"),
            TokenKind::Rcurly => write!(f, "}}"),
            TokenKind::Lbracket => write!(f, "["),
            TokenKind::Rbracket => write!(f, "]"),
            TokenKind::Do => write!(f, "do"),
            TokenKind::End => write!(f, "end"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Unless => write!(f, "unless"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::Trait => write!(f, "trait"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::Export => write!(f, "export"),
            TokenKind::VariableArgs => write!(f, "var"),
            TokenKind::Module => write!(f, "mod"),
            TokenKind::Error => write!(f, "raise"),
            TokenKind::Try => write!(f, "try"),
            TokenKind::Catch => write!(f, "catch"),
            TokenKind::Range => write!(f, ".."),
            TokenKind::RangeInclusive => write!(f, "..="),
            TokenKind::Optional => write!(f, "?"),
            TokenKind::DoubleBang => write!(f, "!!"),
            TokenKind::Comment => write!(f, "# comment"),
            TokenKind::This => write!(f, "self"),
            TokenKind::Arg(a) => write!(f, "${}", a),
            TokenKind::Increment => write!(f, "++"),
            TokenKind::Decrement => write!(f, "--"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Token<'lex> {
    pub(crate) kind: TokenKind<'lex>,
    pub(crate) span: Span,
    pub(crate) line: usize,
}

// todo custom debug impl

impl<'lex> Token<'lex> {
    pub(crate) fn terminal(&self) -> bool {
        self.kind == TokenKind::Newline || self.kind == TokenKind::Semi
    }
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
                TokenKind::Identifier("a"),
                TokenKind::Assign,
                TokenKind::Value(TokenValue::Number(1.into())),
                TokenKind::Identifier("b"),
                TokenKind::Assign,
                TokenKind::Value(TokenValue::Number(2.into())),
                TokenKind::Identifier("a"),
                TokenKind::BinOp(BinaryOperation::Add),
                TokenKind::Identifier("b"),
            ]
        )
    }
}
