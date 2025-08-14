use rigz_ast::*;
use rigz_ast_derive::derive_module;
use rigz_core::*;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

derive_module! {
    r#"trait JSON
        fn Any.to_json -> String!
        fn parse(input: String) -> Any!
    end"#
}

impl RigzJSON for JSONModule {
    #[inline]
    fn any_to_json(&self, value: &ObjectValue) -> Result<String, VMError> {
        match serde_json::to_string(value) {
            Ok(s) => Ok(s),
            Err(e) => Err(VMError::runtime(format!("Failed to write json - {e}"))),
        }
    }

    #[inline]
    fn parse(&self, input: String) -> Result<ObjectValue, VMError> {
        match serde_json::from_str(input.as_str()) {
            Ok(mut v) => {
                if let ObjectValue::Object(o) = &mut v {
                    o.post_deserialize();
                }
                Ok(v)
            }
            Err(e) => Err(VMError::runtime(format!("Failed to parse json - {e}"))),
        }
    }
}
