use rigz_ast::*;
use rigz_ast_derive::derive_module;

derive_module!(
    r#"
import trait Number
    fn Number.ceil -> Number
    fn Number.round -> Number
    fn Number.trunc -> Number
end
"#
);

impl RigzNumber for NumberModule {
    fn number_ceil(&self, this: Number) -> Number {
        match this {
            Number::Int(_) => this,
            Number::Float(f) => (f.ceil() as i64).into(),
        }
    }

    fn number_round(&self, this: Number) -> Number {
        match this {
            Number::Int(_) => this,
            Number::Float(f) => (f.round() as i64).into(),
        }
    }

    fn number_trunc(&self, this: Number) -> Number {
        match this {
            Number::Int(_) => this,
            Number::Float(f) => (f.trunc() as i64).into(),
        }
    }
}
