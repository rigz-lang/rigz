use rigz_ast_derive::derive_module;

derive_module! {
    r#"trait NoneObject
    end"#
}

impl RigzNoneObject for NoneObjectModule {

}