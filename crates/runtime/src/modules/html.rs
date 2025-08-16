use rigz_ast::*;
use rigz_ast_derive::{derive_module, derive_object};
use rigz_core::*;
use scraper::Selector;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

derive_object! {
    "Html",
    struct Html {
        pub document: String,
    },
    r#"object Html
        Self(document: String)

        fn Self.element(selector: String) -> String!?
        fn Self.elements(var id, selectors: String) -> Map
    end"#
}

impl HtmlObject for Html {
    fn element(&self, selector: String) -> Result<Option<String>, VMError> {
        let s = match Selector::parse(&selector) {
            Ok(s) => s,
            Err(e) => {
                return Err(VMError::runtime(format!(
                    "Invalid selector {selector} - {e}"
                )))
            }
        };

        let html = scraper::Html::parse_document(self.document.as_str());
        match html.select(&s).next() {
            None => Ok(None),
            Some(s) => Ok(Some(s.inner_html())),
        }
    }

    fn elements(
        &self,
        ids: Vec<Rc<RefCell<ObjectValue>>>,
        selectors: Vec<String>,
    ) -> IndexMap<ObjectValue, ObjectValue> {
        html_elements(&self.document, ids, selectors)
    }
}

impl AsPrimitive<ObjectValue, Rc<RefCell<ObjectValue>>> for Html {}

impl CreateObject for Html {
    fn create(args: RigzArgs) -> Result<Self, VMError>
    where
        Self: Sized,
    {
        let [html] = args.take()?;
        let html = html.borrow();
        let document = html.to_string();
        Ok(Html { document })
    }
}

fn html_elements(
    this: &str,
    ids: Vec<Rc<RefCell<ObjectValue>>>,
    selectors: Vec<String>,
) -> IndexMap<ObjectValue, ObjectValue> {
    let html = scraper::Html::parse_document(this);
    ids.into_iter()
        .zip(selectors)
        .map(|(id, selector)| {
            let v = match Selector::parse(&selector) {
                Ok(s) => {
                    let select = html.select(&s);
                    let mut res: Vec<_> = select.map(|s| s.inner_html()).collect();
                    match res.len() {
                        0 => ObjectValue::default(),
                        1 => res.remove(0).into(),
                        _ => res.into(),
                    }
                }
                Err(e) => VMError::runtime(format!("Invalid selector {selector}: {e}")).into(),
            };
            (id.borrow().clone(), v)
        })
        .collect()
}

derive_module! {
    [Html],
    r#"trait Html
    fn String.html -> Html::Html = Html::Html.new self
    fn String.elements(var id, selector: String) -> Map
end"#
}

impl RigzHtml for HtmlModule {
    fn string_elements(
        &self,
        this: &String,
        ids: Vec<Rc<RefCell<ObjectValue>>>,
        selectors: Vec<String>,
    ) -> IndexMap<ObjectValue, ObjectValue> {
        html_elements(this, ids, selectors)
    }
}
