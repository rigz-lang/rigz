use rigz_ast::*;
use rigz_ast_derive::derive_module;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub struct HttpModule {
    client: reqwest::Client,
}

derive_module! {
    HttpModule,
    r#"trait Http
        fn get(path, content_type: String? = none) -> String
    end"#
}

impl RigzHttp for HttpModule {
    fn get(&self, path: Value, content_type: Option<String>) -> String {
        todo!()
    }
}
