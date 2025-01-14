use crate::token::TokenKind;
use crate::ParsingError;
use logos::Logos;

pub fn format(input: String) -> String {
    let read = input.as_str().trim();

    if read.is_empty() {
        return input;
    }

    let mut result = String::with_capacity(read.len());
    let mut tokens = TokenKind::lexer(read);
    let mut indent = 0;
    let mut function_scope = false;
    let mut last = TokenKind::Newline;

    while let Some(next) = tokens.next() {
        let token = match next {
            Ok(t) => t,
            Err(_) => {
                // Include invalid output unchanged
                result.push_str(tokens.slice());
                continue;
            }
        };

        match &token {
            TokenKind::Newline => {
                result.push('\n');
            }
            TokenKind::Value(v) => {
                if matches!(
                    last,
                    TokenKind::Assign | TokenKind::BinOp(_) | TokenKind::Colon
                ) {
                    result.push(' ');
                }
                if last == TokenKind::Newline {
                    result.push_str("  ".repeat(indent).as_str());
                }
                if function_scope && matches!(last, TokenKind::Identifier(_)) {
                    result.push('\n');
                    result.push_str("  ".repeat(indent).as_str());
                }
                result.push_str(v.to_string().as_str());
            }
            TokenKind::Assign => {
                result.push_str(" = ");
            }
            TokenKind::Semi => {
                result.push(';');
                result.push('\n');
            }
            TokenKind::Colon => {
                result.push(':');
            }
            TokenKind::Arrow => {
                result.push_str(" -> ");
            }
            TokenKind::Let => {
                result.push_str("let ");
            }
            TokenKind::Mut => {
                result.push_str("mut ");
            }
            TokenKind::BinOp(op) => {
                result.push(' ');
                result.push_str(op.to_string().as_str());
                result.push(' ');
            }
            TokenKind::FunctionDef => {
                result.push_str("fn ");
                function_scope = true;
                indent += 1;
            }
            TokenKind::If | TokenKind::Unless => {
                result.push_str(token.to_string().as_str());
                result.push(' ');
                indent += 1;
            }
            TokenKind::Else => {
                result.push_str("else ");
            }
            TokenKind::Do => {
                result.push_str("do\n");
                indent += 1;
                result.push_str(" ".repeat(indent * 2).as_str());
            }
            TokenKind::Rcurly | TokenKind::Rparen | TokenKind::Rbracket => {
                if indent > 0 {
                    indent -= 1;
                }
                result.push_str(token.to_string().as_str());
            }
            TokenKind::Lcurly | TokenKind::Lparen | TokenKind::Lbracket => {
                result.push_str(token.to_string().as_str());
            }
            TokenKind::Comment => {
                result.push_str("\n");
                result.push_str(" ".repeat(indent * 2).as_str());
                result.push_str(tokens.slice());
            }
            TokenKind::End => {
                if indent > 0 {
                    indent -= 1;
                }
                if last != TokenKind::Newline {
                    result.push('\n');
                    result.push_str("  ".repeat(indent).as_str());
                }
                result.push_str("end");
                function_scope = false;
            }
            _ => {
                result.push_str(token.to_string().as_str());
            }
        }
        last = token;
    }

    result.trim().to_string()
}

#[cfg(test)]
pub mod tests {
    use crate::format;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test(unsupported = test)]
    fn test_format() {
        let input = r#"fn foo
            123
        end"#;
        let formatted = format(input.to_string());
        assert_eq!(formatted, "fn foo\n  123\nend");
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn test_format_one_line() {
        let input = r#"fn foo 123 end"#;
        let formatted = format(input.to_string());
        assert_eq!(formatted, "fn foo\n  123\nend");
    }
}
