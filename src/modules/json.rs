use rigz_ast::*;
use rigz_ast_derive::derive_module;

derive_module!(
    r#"trait JSON
        fn Any.to_json -> String!
        fn parse(input: String) -> Any!
    end"#
);

impl RigzJSON for JSONModule {
    #[inline]
    fn any_to_json(&self, value: Value) -> Result<String, VMError> {
        match serde_json::to_string(&value) {
            Ok(s) => Ok(s),
            Err(e) => Err(VMError::RuntimeError(format!("Failed to write json - {e}"))),
        }
    }

    #[inline]
    fn parse(&self, input: String) -> Result<Value, VMError> {
        match serde_json::from_str(input.as_str()) {
            Ok(v) => Ok(v),
            Err(e) => Err(VMError::RuntimeError(format!("Failed to parse json - {e}"))),
        }
    }
}
