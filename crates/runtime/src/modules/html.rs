use rigz_ast::*;
use rigz_ast_derive::{derive_module, derive_object};
use rigz_core::*;
use scraper::Selector;
use std::cell::RefCell;
use std::rc::Rc;

derive_object! {
    "Html",
    struct Fragment {
        pub partial: String,
    },
    r#"object Fragment
        Self(partial: String)
        fn Self.to_html -> Html::Html
    end"#
}

impl FragmentObject for Fragment {
    fn to_html(&self) -> ObjectValue {
        ObjectValue::new(Html {
            document: self.partial.clone(),
        })
    }
}

impl AsPrimitive<ObjectValue> for Fragment {}

impl CreateObject for Fragment {
    fn create(args: RigzArgs) -> Result<Self, VMError>
    where
        Self: Sized,
    {
        let [html] = args.take()?;
        let html = html.borrow();
        Ok(Fragment {
            partial: html.to_string(),
        })
    }
}

derive_object! {
    "Html",
    struct Html {
        pub document: String,
    },
    r#"object Html
        Self(document: String)

        fn Self.element(selector: String) -> Html::Fragment?
        fn Self.elements(var selector: String) -> Map
    end"#
}

impl HtmlObject for Html {
    fn element(&self, selector: String) -> Option<ObjectValue> {
        todo!()
    }

    fn elements(&self, selector: String) -> IndexMap<ObjectValue, ObjectValue> {
        todo!()
    }
}

impl AsPrimitive<ObjectValue> for Html {}

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

derive_module! {
    [Fragment, Html],
    r#"trait Html
    fn String.html -> Html::Html = Html::Html.new self
    fn String.elements(var id, selector: String) -> Map
end"#
}

impl RigzHtml for HtmlModule {
    fn string_elements(
        &self,
        this: String,
        ids: Vec<ObjectValue>,
        selectors: Vec<String>,
    ) -> IndexMap<ObjectValue, ObjectValue> {
        let html = scraper::Html::parse_document(&this);
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
                    Err(e) => {
                        VMError::RuntimeError(format!("Invalid selector {selector}: {e}")).into()
                    }
                };
                (id, v)
            })
            .collect()
    }
}
