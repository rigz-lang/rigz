use rigz_ast::*;
use rigz_ast_derive::derive_module;
use std::ops::Deref;

#[derive(Debug, Default)]
pub struct HttpModule {
    client: reqwest::blocking::Client,
}

// todo once {String, Value} syntax is supported update these to allow passing in map for headers

derive_module! {
    HttpModule,
    r#"trait Http
        fn get(path: String, content_type: String? = none) -> String!
        fn post(path: String, body: String? = none, content_type: String? = none) -> (String!, String!?)
    end"#
}

// todo once object type is implemented convert response to an object to allow direct access

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

    fn post(&self, path: String, body: Option<String>, content_type: Option<String>) -> (Result<String, VMError>, Result<Option<String>, VMError>) {
        let mut req = self.client.post(path);
        if let Some(content_type) = content_type {
            req = req.header("Content-Type", content_type);
        };
        if let Some(body) = body {
            req = req.body(body);
        }
        match req.send() {
            Ok(t) => {
                let l = match t.headers().get(reqwest::header::LOCATION).map(|n| n.to_str().map(|s| s.to_string())) {
                    Some(Ok(l)) => Ok(Some(l)),
                    None => Ok(None),
                    Some(Err(e)) => Err(VMError::RuntimeError(format!("Failed to convert Location Header to string: {e}"))),
                };
                match t.text() {
                    Ok(t) => (Ok(t), l),
                    Err(e) => (Err(VMError::RuntimeError(format!(
                        "Failed to convert response to text - {e}"
                    ))), l)
                }
            },
            Err(e) => {
                let err = VMError::RuntimeError(format!("Request Failed: {e}"));
                (Err(err.clone()), Err(err))
            },
        }
    }
}
