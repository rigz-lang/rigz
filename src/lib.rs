use logos::{Lexer, Logos, Span};
use rigz_vm::{VMBuilder, VM};

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
enum Token {
    #[token("none")]
    None,
    #[token("=")]
    Eq,
    #[token("!=")]
    Neq,
    #[token("!")]
    Not,
    #[token("<")]
    Lt,
    #[token("<=")]
    Lte,
    #[token(">")]
    Gt,
    #[token(">=")]
    Gte,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("&")]
    BitAnd,
    #[token("|")]
    BitOr,
    #[token("^")]
    Xor,
    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("%")]
    Rem,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,
    #[token(".")]
    Period,
    #[regex("[A-Za-z_]+", priority=3)]
    Identifier,
    #[regex("[A-Za-z_$]+")]
    FunctionIdentifier,
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
    #[regex("-?\\d+(\\.\\d+)?")]
    Number,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("'")]
    SingleQuote,
    #[token("\"")]
    DoubleQuote,
    #[token("`")]
    Tick,
}

pub struct Bucket<'vm> {
    token: Token,
    span: Span,
    slice: &'vm str
}

pub struct StateMachine<'vm> {
    builder: VMBuilder<'vm>,
    lexer: Lexer<'vm, Token>,
    current: Bucket<'vm>,
    needs_token: Vec<Bucket<'vm>>
}

impl<'vm> StateMachine<'vm> {
    pub fn vm(&mut self) -> VM {
        self.builder.build()
    }

    fn lex_next(lexer: &mut Lexer<'vm, Token>) -> Option<Bucket> {
        match lexer.next() {
            None => None,
            Some(t) => {
                let span = lexer.span();
                let slice = lexer.slice();
                Some(Bucket {
                    token: t.expect("Failed to parse token"),
                    span,
                    slice,
                })
            }
        }
    }

    pub fn new(input: &str) -> Self {
        let mut lexer = Token::lexer(input);
        let current = Self::lex_next(&mut lexer).expect("Attempted to lex no tokens");
        Self {
            lexer,
            builder: VMBuilder::new(),
            current,
            needs_token: vec![]
        }
    }

    pub fn next(&mut self, bucket: Bucket) {

    }

    pub fn next_token(&mut self) -> Option<Bucket> {
        Self::lex_next(&mut self.lexer)
    }
}

fn internal_parse(input: &str) -> StateMachine {
    let mut fsm = StateMachine::new(input);
    loop {
        match fsm.next_token() {
            None => break,
            Some(t) => fsm.next(t),
        }
    }
    fsm
}

pub fn parse(input: &str) -> VM {
    let mut fsm = internal_parse(input);
    fsm.vm()
}

#[cfg(test)]
mod tests {
    use rigz_vm::Scope;
    use super::*;

    #[test]
    fn tokenize_works() {
        let raw = r#"
            a = 1
            b = 2
            a + b
        "#;
        // current (Identifier, a, 0..1)
        // current (Eq, =, 2..3), stack: {identifier}
        // current (Number, 1, 4..5), stack: {Eq, identifier}
        // vm.

        let lexer = Token::lexer(raw);
        let actual: Vec<Token> = lexer.map(|t| t.unwrap()).collect();
        assert_eq!(actual, vec![
            Token::Identifier,
            Token::Eq,
            Token::Number,
            Token::Identifier,
            Token::Eq,
            Token::Number,
            Token::Identifier,
            Token::Add,
            Token::Identifier,
        ])
    }

    #[test]
    fn create_vm_works() {
        let a = "puts 'Hello World'";
        let fsm = internal_parse(a);
        let scope = Scope::new();
        assert_eq!(vec![], fsm.builder.scopes)
    }
}
