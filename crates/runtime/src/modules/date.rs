use rigz_ast::*;
use rigz_ast_derive::derive_module;

derive_module!(
    r#"
trait Date
    fn now -> Number
    fn utc -> Number
end
"#
);

// todo use object instead of Number
impl RigzDate for DateModule {
    fn now(&self) -> Number {
        chrono::Local::now().timestamp_millis().into()
    }

    fn utc(&self) -> Number {
        chrono::Utc::now().timestamp_millis().into()
    }
}
