use std::ops::Deref;
use rigz_ast::*;
use rigz_ast_derive::derive_module;
use rigz_core::*;
use std::cell::RefCell;
use std::rc::Rc;

// need to borrow this for ext
derive_module!(
    r#"trait JSON
        fn Any.to_json -> String!
        fn parse(input: String) -> Any!
    end"#
);

#[allow(unused_variables)]
impl RigzJSON for JSONModule {
    fn any_to_json(&self, value: &ObjectValue) -> Result<String, VMError> {
        match serde_json::to_string(&value) {
            Ok(s) => Ok(s),
            Err(e) => Err(VMError::runtime(format!("Failed to write json - {e}"))),
        }
    }

    fn parse(&self, input: String) -> Result<ObjectValue, VMError> {
        match serde_json::from_str(input.as_str()) {
            Ok(v) => Ok(v),
            Err(e) => Err(VMError::runtime(format!("Failed to parse json - {e}"))),
        }
    }
}

use wasm_bindgen_test::*;

#[wasm_bindgen_test(unsupported = test)]
fn blah() {
    let json = JSONModule;
    assert_eq!(
        "5",
        json.call("parse", vec![Rc::new(RefCell::new(5.into()))].into())
            .expect("json parse failed")
            .to_string()
            .as_str()
    )
}
