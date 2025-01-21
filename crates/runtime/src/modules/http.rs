use rigz_ast::*;
use rigz_ast_derive::{derive_module, derive_object};
use rigz_core::*;
use std::ops::Deref;

// pub struct Response {
//
// }
//
// derive_object! {
//     "Http",
//     Response,
//     r#"object Response
//     end"#
// }

#[derive(Debug)]
pub struct HttpModule {
    client: ureq::Agent,
}

impl Default for HttpModule {
    fn default() -> Self {
        Self {
            client: ureq::Agent::new(),
        }
    }
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
        let mut req = self.client.get(&path);
        if let Some(content_type) = content_type {
            req = req.set("Content-Type", &content_type);
        }

        match req.call().map(|r| r.into_string()) {
            Ok(Ok(t)) => Ok(t),
            Ok(Err(t)) => Err(VMError::RuntimeError(format!(
                "Failed to convert response to text - {t}"
            ))),
            Err(e) => Err(VMError::RuntimeError(format!("Request Failed: {e}"))),
        }
    }

    fn post(
        &self,
        path: String,
        body: Option<String>,
        content_type: Option<String>,
    ) -> (Result<String, VMError>, Result<Option<String>, VMError>) {
        let mut req = self.client.post(&path);
        if let Some(content_type) = content_type {
            req = req.set("Content-Type", &content_type);
        };
        let resp = if let Some(body) = body {
            req.send_string(&body)
        } else {
            req.call()
        };
        match resp {
            Ok(t) => {
                let l = match t.header("Location").map(|n| n.to_string()) {
                    Some(l) => Ok(Some(l)),
                    None => Ok(None),
                };
                match t.into_string() {
                    Ok(t) => (Ok(t), l),
                    Err(e) => (
                        Err(VMError::RuntimeError(format!(
                            "Failed to convert response to text - {e}"
                        ))),
                        l,
                    ),
                }
            }
            Err(e) => {
                let err = VMError::RuntimeError(format!("Request Failed: {e}"));
                (Err(err.clone()), Err(err))
            }
        }
    }
}
