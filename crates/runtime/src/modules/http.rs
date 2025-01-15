use rigz_ast::*;
use rigz_ast_derive::derive_module;
use std::ops::Deref;

#[derive(Debug, Default)]
pub struct HttpModule {
    client: reqwest::blocking::Client,
}

derive_module! {
    HttpModule,
    r#"trait Http
        fn get(path: String, content_type: String? = none) -> String!
    end"#
}

impl RigzHttp for HttpModule {
    fn get(&self, path: String, content_type: Option<String>) -> Result<String, VMError> {
        let mut req = self.client.get(path);
        if let Some(content_type) = content_type {
            req = req.header(reqwest::header::CONTENT_TYPE, content_type);
        }

        match req.send().map(|r| r.text()) {
            Ok(Ok(t)) => Ok(t),
            Ok(Err(t)) => Err(VMError::RuntimeError(format!(
                "Failed to convert response to text - {t}"
            ))),
            Err(e) => Err(VMError::RuntimeError(format!("Request Failed: {e}"))),
        }
    }
}
