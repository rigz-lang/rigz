use rigz_ast::*;
use rigz_ast_derive::derive_module;
use rigz_core::*;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

derive_module! {
    r#"import trait String
    fn mut String.push(value)
    fn String.concat(value: String) -> String
    fn String.with(var value) -> String
    fn String.trim -> String
    fn String.lines -> [String]
    fn String.split(pattern: String) -> [String]
    fn String.replace(pattern: String, value: String) -> String
end"#
}

impl RigzString for StringModule {
    fn mut_string_push(&self, this: &mut String, value: Rc<RefCell<ObjectValue>>) {
        this.push_str(value.borrow().to_string().as_str())
    }

    fn string_concat(&self, this: &String, value: String) -> String {
        let mut this = this.clone();
        this.push_str(value.to_string().as_str());
        this
    }

    fn string_with(&self, this: &String, value: Vec<Rc<RefCell<ObjectValue>>>) -> String {
        let mut this = this.clone();
        for v in value {
            this.push_str(v.borrow().to_string().as_str())
        }
        this
    }

    fn string_trim(&self, this: &String) -> String {
        this.trim().to_string()
    }

    fn string_lines(&self, this: &String) -> Vec<String> {
        this.lines().map(|s| s.to_string()).collect()
    }

    fn string_split(&self, this: &String, pattern: String) -> Vec<String> {
        this.split(&pattern).map(|s| s.to_string()).collect()
    }

    fn string_replace(&self, this: &String, pattern: String, value: String) -> String {
        this.replace(pattern.as_str(), value.as_str())
    }
}
