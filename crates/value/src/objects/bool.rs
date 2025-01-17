use rigz_ast_derive::derive_module;

pub struct Boolean {
    pub(crate) value: bool,
}

derive_module! {
    Boolean,
    r#"trait Boolean
    end"#
}

impl RigzBoolean for Boolean {

}