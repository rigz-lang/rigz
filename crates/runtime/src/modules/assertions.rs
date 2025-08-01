use rigz_ast::*;
use rigz_ast_derive::derive_module;
use rigz_core::*;

derive_module! {
    r#"
    import trait Assertions
        fn assert(condition: Bool, message = '') -> None!
        fn assert_eq(lhs, rhs, message = '') -> None!
        fn assert_neq(lhs, rhs, message = '') -> None!
    end
"#
}

impl RigzAssertions for AssertionsModule {
    // todo support formatting message
    fn assert(&self, condition: bool, message: String) -> Result<(), VMError> {
        if !condition {
            let message = if message.is_empty() {
                "Assertion Failed".to_string()
            } else {
                format!("Assertion Failed: {message}")
            };
            return Err(VMError::runtime(message));
        }
        Ok(())
    }

    fn assert_eq(
        &self,
        lhs: ObjectValue,
        rhs: ObjectValue,
        message: String,
    ) -> Result<(), VMError> {
        if lhs == rhs {
            return Ok(());
        }

        let base = format!("\tLeft: {lhs}\n\t\tRight: {rhs}");
        let message = if message.is_empty() {
            format!("Assertion Failed\n\t{base}")
        } else {
            format!("Assertion Failed: {message}\n\t{base}")
        };

        Err(VMError::runtime(message))
    }

    fn assert_neq(
        &self,
        lhs: ObjectValue,
        rhs: ObjectValue,
        message: String,
    ) -> Result<(), VMError> {
        if lhs != rhs {
            return Ok(());
        }

        let base = format!("\tLeft: {lhs}\n\t\tRight: {rhs}");
        let message = if message.is_empty() {
            format!("Assertion Failed\n\t{base}")
        } else {
            format!("Assertion Failed: {message}\n\t{base}")
        };

        Err(VMError::runtime(message))
    }
}
