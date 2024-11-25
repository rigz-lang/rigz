use rigz_ast::*;
use rigz_ast_derive::derive_module;
use uuid::Uuid;

derive_module!(
    r#"
trait UUID
    fn v4 -> String
    fn from(input: String) -> String!
end
"#
);

// todo once object exists use that instead of strings
impl RigzUUID for UUIDModule {
    fn v4(&self) -> String {
        Uuid::new_v4().to_string()
    }

    fn from(&self, input: String) -> Result<String, VMError> {
        match Uuid::try_parse(input.as_str()) {
            Ok(s) => Ok(s.to_string()),
            Err(e) => Err(VMError::RuntimeError(format!("Failed to parse uuid: {e}"))),
        }
    }
}
