use rigz_ast_derive::derive_module;
use rigz_core::ObjectValue;

#[derive(Clone)]
pub struct AnyObject {
    value: ObjectValue,
}

derive_module! {
    AnyObject,
    r#"trait Any
    end"#
}

impl RigzAny for AnyObject {

}