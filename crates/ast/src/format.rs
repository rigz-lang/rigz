use crate::token::TokenKind;
use logos::{Lexer, Logos};

struct Formmatter<'l> {
    result: String,
    indent: usize,
    last: TokenKind<'l>,
    needs_args: bool,
}

impl<'l> Formmatter<'l> {
    fn new(len: usize) -> Self {
        Self {
            result: String::with_capacity(len),
            indent: 0,
            last: TokenKind::Newline,
            needs_args: false,
        }
    }

    fn format(mut self, mut tokens: Lexer<'l, TokenKind<'l>>) -> String {
        while let Some(next) = tokens.next() {
            let Ok(token) = next else {
                // Include invalid output unchanged
                self.result.push_str(tokens.slice());
                continue;
            };

            if self.needs_args {
                if !matches!(
                    token,
                    TokenKind::Lparen | TokenKind::Assign | TokenKind::Newline
                ) {
                    self.last = TokenKind::Newline;
                    self.result.push('\n');
                }

                if token == TokenKind::Assign {
                    self.indent = self.indent.saturating_sub(1);
                }
                self.needs_args = false;
            } else if matches!(self.last, TokenKind::Do | TokenKind::Catch)
                && token == TokenKind::Assign
            {
                self.indent = self.indent.saturating_sub(1);
            }

            if token == TokenKind::Newline {
                self.result.push('\n')
            } else {
                self.new_indent(token);
                self.leading_spaces(token);
                if token == TokenKind::Comment {
                    self.result.push_str(tokens.slice())
                } else {
                    self.result.push_str(token.to_string().as_str())
                }
            }
            self.last = token;
        }
        self.result
    }

    fn leading_spaces(&mut self, next: TokenKind<'l>) {
        if matches!(
            next,
            TokenKind::Comma
                | TokenKind::Lparen
                | TokenKind::Rparen
                | TokenKind::Lbracket
                | TokenKind::Rbracket
        ) || matches!(self.last, TokenKind::Lparen | TokenKind::Lbracket)
        {
            return;
        }

        if self.last == TokenKind::Newline {
            self.result.push_str(" ".repeat(self.indent * 2).as_str());
            return;
        }

        let char = if next == TokenKind::End { '\n' } else { ' ' };
        self.result.push(char)
    }

    fn new_indent(&mut self, next: TokenKind<'l>) {
        match next {
            TokenKind::Identifier(_) if self.last == TokenKind::FunctionDef => {
                self.needs_args = true;
                self.indent += 1
            }
            // todo handle if/unless/else
            TokenKind::Do | TokenKind::Catch => self.indent += 1,
            TokenKind::End => self.indent = self.indent.saturating_sub(1),
            _ => {}
        }
    }
}

pub fn format(input: String) -> String {
    let read = input.as_str();

    Formmatter::new(read.len()).format(TokenKind::lexer(read))
}

macro_rules! test_format {
    ($($id: ident: $input: literal = $output: literal;)*) => {
        #[cfg(test)]
        pub mod formatting {
            use wasm_bindgen_test::wasm_bindgen_test;

            $(
                #[wasm_bindgen_test(unsupported = test)]
                fn $id() {
                    assert_eq!(crate::format($input.to_string()), $output.to_string())
                }
            )*
        }
    };
}

test_format! {
    basic: "2 + 2" = "2 + 2";
    incomplete: "2 +" = "2 +";
    single_line_if: "foo if bar" = "foo if bar";
    single_line_if_spaces: "foo      unless      bar" = "foo unless bar";
    single_line_fn: "fn foo 123 end" = "fn foo\n  123\nend";
    single_line_fn_eq: "fn foo = 123" = "fn foo = 123";
    single_line_fn_eq_args: "fn foo(a,b,c)=a+b+c" = "fn foo(a, b, c) = a + b + c";
    list: "[1,2,3]" = "[1, 2, 3]";
    map: "{1,2,3}" = "{ 1, 2, 3 }";
    multi_line_fn: "fn foo\n123+bar\nend" = "fn foo\n  123 + bar\nend";
    multi_line_fn_id: "fn foo\nbar+baz\nend" = "fn foo\n  bar + baz\nend";
    preserve_new_lines_comment: "\n\n\n\n#hello world\n\n\n" = "\n\n\n\n#hello world\n\n\n";
    eat_whitespace: "     " = "";
    revert_indent_after_single_fn: "fn foo = 123\nfn bar = baz" = "fn foo = 123\nfn bar = baz";
}
