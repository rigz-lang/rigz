# rigz_ast
Generate an AST for a given input.

## Usage
```rust
// Create a parser
fn parse() {
    let input = "2 + 2";
    let mut parser = Parser::prepare(input).expect("Invalid Tokens");
    let program = parser.parse().expect("Failed to parse");
}
```

