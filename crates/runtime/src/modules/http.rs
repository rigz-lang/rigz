use log::warn;
use rigz_ast::*;
use rigz_ast_derive::{derive_module, derive_object};
use rigz_core::*;
use std::ops::Deref;
use std::sync::{Arc, LazyLock, RwLock};

derive_object! {
    "Http",
    struct Request {
        pub method: Option<String>,
        pub path: String,
        pub body: Option<ObjectValue>,
        #[derivative(Hash="ignore", PartialEq="ignore", PartialOrd="ignore")]
        pub headers: Option<IndexMap<ObjectValue, ObjectValue>>,
    },
    r#"object Request
        Self(path: String, method: String? = none, body: Any? = none, headers: Map? = none)

        fn Self.header(key) -> Any?

        fn mut Self.body(body: Any)
        fn mut Self.method(method: String)
        fn mut Self.headers(var key: String, value: String)
    end"#
}

impl Request {
    fn from_map(map: IndexMap<ObjectValue, ObjectValue>) -> Result<Self, VMError> {
        let path = match map.get(&ObjectValue::Primitive(PrimitiveValue::String(
            "path".to_string(),
        ))) {
            None => {
                return Err(VMError::runtime(format!(
                    "Missing `path`, cannot create request from {map:?}"
                )))
            }
            Some(p) => p.to_string(),
        };

        let method = map
            .get(&ObjectValue::Primitive(PrimitiveValue::String(
                "method".to_string(),
            )))
            .map(|o| o.to_string());
        let body = map
            .get(&ObjectValue::Primitive(PrimitiveValue::String(
                "body".to_string(),
            )))
            .cloned();

        let headers = match map.get(&ObjectValue::Primitive(PrimitiveValue::String(
            "headers".to_string(),
        ))) {
            None => None,
            Some(p) => match p {
                ObjectValue::Map(m) => Some(m.clone()),
                ObjectValue::Object(o) => Some(o.to_map()?),
                o => {
                    return Err(VMError::runtime(format!(
                        "Cannot convert {o} to Map<String, String>"
                    )))
                }
            },
        };

        Ok(Request {
            method,
            path,
            body,
            headers,
        })
    }
}

impl AsPrimitive<ObjectValue> for Request {}

impl RequestObject for Request {
    fn header(&self, key: ObjectValue) -> Option<ObjectValue> {
        self.headers.as_ref().map(|h| h.get(&key).cloned())?
    }

    fn mut_body(&mut self, body: ObjectValue) {
        self.body = Some(body);
    }

    fn mut_method(&mut self, method: String) {
        self.method = Some(method);
    }

    fn mut_headers(&mut self, key: Vec<String>, value: Vec<String>) {
        if self.headers.is_none() {
            self.headers = Some(IndexMap::new());
        }
        match &mut self.headers {
            None => unreachable!(),
            Some(h) => {
                for (k, v) in key.into_iter().zip(value) {
                    h.insert(k.into(), v.into());
                }
            }
        }
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
            let [path, method, body, headers] = args.take()?;
            let headers = match headers.borrow().map(|o| o.to_map()) {
                None => None,
                Some(Ok(s)) => Some(s),
                Some(Err(e)) => return Err(e),
            };
            let method = method.borrow();
            let body = body.borrow();
            let path = path.borrow();
            Ok(Self {
                path: path.to_string(),
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
        #[serde(skip)]
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
                return Err(VMError::runtime(format!(
                    "Failed to get response value - {e}"
                )))
            }
        };

        match v.take() {
            None => Err(VMError::runtime("Response has been consumed".to_string())),
            Some(r) => match r.into_string() {
                Ok(s) => Ok(s),
                Err(e) => Err(VMError::runtime(format!(
                    "Failed to convert response to string - {e}"
                ))),
            },
        }
    }
}

#[derive(Debug)]
pub struct HttpModule {
    client: LazyLock<ureq::Agent>,
}

impl Default for HttpModule {
    fn default() -> Self {
        Self {
            client: LazyLock::new(ureq::Agent::new),
        }
    }
}

derive_module! {
    HttpModule,
    [Request, Response],
    r#"trait Http
        fn request -> Http::Request = Http::Request.new
        fn fetch(request: Http::Request) -> Http::Response!
        fn head(path: String, headers: Map? = none) -> Http::Response!
        fn get(path: String, headers: Map? = none) -> Http::Response!
        fn delete(path: String, headers: Map? = none) -> Http::Response!
        fn options(path: String, headers: Map? = none) -> Http::Response!
        fn patch(path: String, body: Any? = none, headers: Map? = none) -> Http::Response!
        fn post(path: String, body: Any? = none, headers: Map? = none) -> Http::Response!
        fn put(path: String, body: Any? = none, headers: Map? = none) -> Http::Response!
    end"#
}

fn set_headers(
    request: ureq::Request,
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
        Err(e) => Err(VMError::runtime(format!("Request Failed: {e}"))),
    }
}

fn handle_body(req: ureq::Request, body: Option<ObjectValue>) -> Result<ObjectValue, VMError> {
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

impl RigzHttp for HttpModule {
    fn fetch(&self, request: ObjectValue) -> Result<ObjectValue, VMError> {
        let r: Request = match request {
            ObjectValue::Map(m) => Request::from_map(m)?,
            ObjectValue::Object(o) => match o.downcast_ref::<Request>() {
                Some(r) => r.clone(),
                None => {
                    return Err(VMError::UnsupportedOperation(format!(
                        "Cannot convert {o} to Http::Request"
                    )))
                }
            },
            o => return Err(VMError::todo(format!("`fetch` cannot be called with {o}"))),
        };
        match r.method {
            None => {
                if r.body.is_some() {
                    warn!("Ignoring body for GET request - {:?}", r.body)
                }
                self.get(r.path, r.headers)
            }
            Some(s) => match s.to_lowercase().as_str() {
                "get" => {
                    if r.body.is_some() {
                        warn!("Ignoring body for GET request - {:?}", r.body)
                    }
                    self.get(r.path, r.headers)
                }
                "delete" => {
                    if r.body.is_some() {
                        warn!("Ignoring body for DELETE request - {:?}", r.body)
                    }
                    self.delete(r.path, r.headers)
                }
                "post" => self.post(r.path, r.body, r.headers),
                "put" => self.post(r.path, r.body, r.headers),
                method => Err(VMError::runtime(format!("Invalid HTTP method {method}"))),
            },
        }
    }

    fn head(
        &self,
        path: String,
        headers: Option<IndexMap<ObjectValue, ObjectValue>>,
    ) -> Result<ObjectValue, VMError> {
        let mut req = self.client.head(&path);
        req = set_headers(req, headers);

        let res = req.call();
        to_object(res)
    }

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

    fn options(
        &self,
        path: String,
        headers: Option<IndexMap<ObjectValue, ObjectValue>>,
    ) -> Result<ObjectValue, VMError> {
        let mut req = self.client.request("OPTIONS", &path);
        req = set_headers(req, headers);

        let res = req.call();
        to_object(res)
    }

    fn patch(
        &self,
        path: String,
        body: Option<ObjectValue>,
        headers: Option<IndexMap<ObjectValue, ObjectValue>>,
    ) -> Result<ObjectValue, VMError> {
        let mut req = self.client.patch(&path);
        req = set_headers(req, headers);
        handle_body(req, body)
    }

    fn post(
        &self,
        path: String,
        body: Option<ObjectValue>,
        headers: Option<IndexMap<ObjectValue, ObjectValue>>,
    ) -> Result<ObjectValue, VMError> {
        let mut req = self.client.post(&path);
        req = set_headers(req, headers);
        handle_body(req, body)
    }

    fn put(
        &self,
        path: String,
        body: Option<ObjectValue>,
        headers: Option<IndexMap<ObjectValue, ObjectValue>>,
    ) -> Result<ObjectValue, VMError> {
        let mut req = self.client.put(&path);
        req = set_headers(req, headers);
        handle_body(req, body)
    }
}
