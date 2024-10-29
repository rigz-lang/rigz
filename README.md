# rigz_ast_derive

Generate a trait, Module impl, and ParsedModule impl for static rigz input at compile time, otherwise modules are parsed and validated at runtime.

## Example

Shown below is the JSONModule used by the [rigz_runtime](https://crates.io/crates/rigz_runtime).

Functions with a default implementation will not appear in the trait as they are handled by the runtime directly.

```rust
use rigz_ast::*;
use rigz_ast_derive::derive_module;

derive_module!(
    r#"trait JSON
        fn Any.to_json -> String!
        fn parse(input: String) -> Any!
    end"#
);

impl RigzJSON for JSONModule {
    fn any_to_json(&self, value: Value) -> Result<String, VMError> {
        match serde_json::to_string(&value) {
            Ok(s) => Ok(s),
            Err(e) => Err(VMError::RuntimeError(format!("Failed to write json - {e}"))),
        }
    }

    fn parse(&self, input: String) -> Result<Value, VMError> {
        match serde_json::from_str(input.as_str()) {
            Ok(v) => Ok(v),
            Err(e) => Err(VMError::RuntimeError(format!("Failed to parse json - {e}"))),
        }
    }
}
```