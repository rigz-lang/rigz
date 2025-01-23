use rigz_ast::*;
use rigz_ast_derive::{derive_module, derive_object};
use rigz_core::*;
use std::io::Error;
use std::ops::Deref;
use std::sync::{Arc, LockResult, RwLock};

derive_object! {
    "Http",
    struct Request {
        pub method: Option<String>,
        pub body: Option<ObjectValue>,
        #[derivative(Hash="ignore", PartialEq="ignore", PartialOrd="ignore")]
        pub headers: Option<IndexMap<ObjectValue, ObjectValue>>,
    },
    r#"object Request
        Self(method: String? = none, body: Any? = none, headers: Map? = none)

        fn Self.header(key) -> Any?

        fn mut Self.body(body: Any)
        fn mut Self.method(method: String)
        fn mut Self.headers(var key: String, value: String)
    end"#
}

impl AsPrimitive<ObjectValue> for Request {}

impl RequestObject for Request {
    fn header(&self, key: ObjectValue) -> Option<ObjectValue> {
        todo!()
    }

    fn mut_body(&mut self, body: ObjectValue) {
        todo!()
    }

    fn mut_method(&mut self, method: String) {
        todo!()
    }

    fn mut_headers(&mut self, key: String, value: String) {
        todo!()
    }
}

impl CreateObject for Request {
    fn create(args: RigzArgs) -> Result<Self, VMError>
    where
        Self: Sized,
    {
        if args.is_empty() {
            Ok(Self::default())
        } else {
            let [method, body, headers] = args.take()?;
            let headers = match headers.borrow().map(|o| o.to_map()) {
                None => None,
                Some(Ok(s)) => Some(s),
                Some(Err(e)) => return Err(e),
            };
            let method = method.borrow();
            let body = body.borrow();
            Ok(Self {
                method: method.map(|o| o.to_string()),
                body: body.map(|o| o.clone()),
                headers,
            })
        }
    }
}

derive_object! {
    "Http",
    struct Response {
        #[cfg_attr(feature = "serde", serde(skip))]
        #[derivative(Debug="ignore", Hash="ignore", PartialEq="ignore", PartialOrd="ignore")]
        resp: Arc<RwLock<Option<ureq::Response>>>,
    },
    r#"object Response
        fn mut Self.text -> String!
        fn mut Self.html -> Html::Html!
            text = try self.text
            Html::Html.new text
        end
    end"#
}

impl From<ureq::Response> for Response {
    fn from(value: ureq::Response) -> Self {
        Self {
            resp: Arc::new(Some(value).into()),
        }
    }
}

impl AsPrimitive<ObjectValue> for Response {}

impl CreateObject for Response {
    fn create(args: RigzArgs) -> Result<Self, VMError>
    where
        Self: Sized,
    {
        Err(VMError::UnsupportedOperation(format!(
            "Cannot create Response directly - {args:?}"
        )))
    }
}

impl ResponseObject for Response {
    fn mut_text(&mut self) -> Result<String, VMError> {
        let mut v = match self.resp.write() {
            Ok(v) => v,
            Err(e) => {
                return Err(VMError::RuntimeError(format!(
                    "Failed to get response value - {e}"
                )))
            }
        };

        match v.take() {
            None => Err(VMError::RuntimeError(
                "Response has been consumed".to_string(),
            )),
            Some(r) => match r.into_string() {
                Ok(s) => Ok(s),
                Err(e) => Err(VMError::RuntimeError(format!(
                    "Failed to convert response to string - {e}"
                ))),
            },
        }
    }
}

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

derive_module! {
    HttpModule,
    [Request, Response],
    r#"trait Http
        fn request -> Http::Request = Http::Request.new
        fn get(path: String, headers: Map? = none) -> Http::Response!
        fn delete(path: String, headers: Map? = none) -> Http::Response!
        fn post(path: String, body: Any? = none, headers: Map? = none) -> Http::Response!
        fn put(path: String, body: Any? = none, headers: Map? = none) -> Http::Response!
        fn fetch(request: Http::Request) -> Http::Response!
    end"#
}

fn set_headers(
    mut request: ureq::Request,
    headers: Option<IndexMap<ObjectValue, ObjectValue>>,
) -> ureq::Request {
    let mut request = request;
    if let Some(headers) = headers {
        for (k, v) in headers {
            request = request.set(k.to_string().as_str(), v.to_string().as_str());
        }
    }
    request
}

#[inline]
fn to_object(res: Result<ureq::Response, ureq::Error>) -> Result<ObjectValue, VMError> {
    match res {
        Ok(r) => {
            let resp: Response = r.into();
            Ok(ObjectValue::new(resp))
        }
        Err(e) => Err(VMError::RuntimeError(format!("Request Failed: {e}"))),
    }
}

impl RigzHttp for HttpModule {
    fn get(
        &self,
        path: String,
        headers: Option<IndexMap<ObjectValue, ObjectValue>>,
    ) -> Result<ObjectValue, VMError> {
        let mut req = self.client.get(&path);
        req = set_headers(req, headers);

        let res = req.call();
        to_object(res)
    }

    fn delete(
        &self,
        path: String,
        headers: Option<IndexMap<ObjectValue, ObjectValue>>,
    ) -> Result<ObjectValue, VMError> {
        let mut req = self.client.delete(&path);
        req = set_headers(req, headers);

        let res = req.call();
        to_object(res)
    }

    fn post(
        &self,
        path: String,
        body: Option<ObjectValue>,
        headers: Option<IndexMap<ObjectValue, ObjectValue>>,
    ) -> Result<ObjectValue, VMError> {
        let mut req = self.client.post(&path);
        req = set_headers(req, headers);
        let resp = match body {
            None => req.call(),
            Some(ObjectValue::Primitive(PrimitiveValue::String(body))) => req.send_string(&body),
            // todo support form
            // todo support bytes
            Some(o) => {
                // todo use json
                req.send_string(&o.to_string())
            }
        };
        to_object(resp)
    }

    fn put(
        &self,
        path: String,
        body: Option<ObjectValue>,
        headers: Option<IndexMap<ObjectValue, ObjectValue>>,
    ) -> Result<ObjectValue, VMError> {
        let mut req = self.client.put(&path);
        req = set_headers(req, headers);
        let resp = match body {
            None => req.call(),
            Some(ObjectValue::Primitive(PrimitiveValue::String(body))) => req.send_string(&body),
            // todo support form
            // todo support bytes
            Some(o) => {
                // todo use json
                req.send_string(&o.to_string())
            }
        };
        to_object(resp)
    }

    fn fetch(&self, request: ObjectValue) -> Result<ObjectValue, VMError> {
        Err(VMError::todo(format!(
            "`fetch` is not implemented, {request}"
        )))
    }
}
