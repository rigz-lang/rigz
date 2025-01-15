use rigz_ast::*;
use rigz_ast_derive::derive_module;
use scraper::Selector;
use std::cell::RefCell;
use std::rc::Rc;

derive_module! {
    r#"trait Html
    fn String.elements(var id, selector: String) -> Map
end"#
}

impl RigzHtml for HtmlModule {
    fn string_elements(
        &self,
        this: String,
        ids: Vec<Value>,
        selectors: Vec<String>,
    ) -> IndexMap<Value, Value> {
        let html = scraper::Html::parse_document(&this);
        ids.into_iter()
            .zip(selectors)
            .map(|(id, selector)| {
                let v = match Selector::parse(&selector) {
                    Ok(s) => {
                        let mut select = html.select(&s);
                        let mut res: Vec<_> = select.map(|s| s.inner_html()).collect();
                        match res.len() {
                            0 => Value::None,
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
