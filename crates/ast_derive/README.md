# rigz_ast_derive

**AST Derive is built under the assumption that ObjectValue from [rigz_value](https://crates.io/crates/rigz_value) will be used.**

Generate a trait, Object impl, and ParsedObject impl for static rigz input at compile time, otherwise modules are parsed and validated at runtime.

## Example

Shown below is the JSONModule used by the [rigz_runtime](https://crates.io/crates/rigz_runtime).

Functions with a default implementation will not appear in the trait as they are handled by the runtime directly.

```rust
use rigz_ast::*;
use rigz_ast_derive::derive_module;
// These imports are only required if an extension function is used
use std::rc::Rc;
use std::cell::RefCell;

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
            Err(e) => Err(VMError::runtime(format!("Failed to write json - {e}"))),
        }
    }

    fn parse(&self, input: String) -> Result<Value, VMError> {
        match serde_json::from_str(input.as_str()) {
            Ok(v) => Ok(v),
            Err(e) => Err(VMError::runtime(format!("Failed to parse json - {e}"))),
        }
    }
}
```

If you already have a struct or enum that you'd like to use for the Module you can pass that as the first argument:

```rust
pub struct HttpModule {
  // ...
}

derive_module! {
    HttpModule,
    r#"trait Http
        fn get(path, content_type = "") -> String
    end"#
}
```

## Todo
- Rc<RefCell<Value>> are cloned into Values before calling generated module call, however these should be references leaving it up to the function whether to clone or not. The problem here revolves around mutable extension functions, if the mutable arg and another arg are the same refcell the second borrow will panic.
  - There are three options here:
    - Use try_borrow and return an error
    - Clone all arguments passed into mutable extensions before the mutable borrow occurs
    - Keep cloning and accept that module calls are less efficient than they could be
- Self cannot be used as a return type