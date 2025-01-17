use rigz_ast_derive::derive_module;
use rigz_core::Number;

pub struct NumberObject {
    pub(crate) value: Number
}

derive_module! {
    NumberObject,
    r#"trait NumberObject

    end"#
}

impl NumberObject {

}