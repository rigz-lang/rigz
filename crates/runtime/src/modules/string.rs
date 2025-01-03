use rigz_ast::*;
use rigz_ast_derive::derive_module;
use std::cell::RefCell;
use std::rc::Rc;

derive_module!(
    r#"
import trait String
    fn mut String.push(value)
    fn String.concat(value: String) -> String
    fn String.with(var value) -> String
    fn String.trim -> String
    fn String.split(pattern: String) -> [String]
    fn String.replace(pattern: String, value: String) -> String
end"#
);

impl RigzString for StringModule {
    fn mut_string_push(&self, this: &mut String, value: Value) {
        this.push_str(value.to_string().as_str())
    }

    fn string_concat(&self, this: String, value: String) -> String {
        let mut this = this;
        this.push_str(value.to_string().as_str());
        this
    }

    fn string_with(&self, this: String, value: Vec<Value>) -> String {
        let mut this = this;
        for v in value {
            this.push_str(v.to_string().as_str())
        }
        this
    }

    fn string_trim(&self, this: String) -> String {
        this.trim().to_string()
    }

    fn string_split(&self, this: String, pattern: String) -> Vec<String> {
        this.split(&pattern).map(|s| s.to_string()).collect()
    }

    fn string_replace(&self, this: String, pattern: String, value: String) -> String {
        this.replace(pattern.as_str(), value.as_str())
    }
}
