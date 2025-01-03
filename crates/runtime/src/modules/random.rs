use rigz_ast::*;
use rigz_ast_derive::derive_module;

derive_module!(
    r#"trait Random
        fn next_int -> Int
        fn next_float -> Float
        fn next_bool(percent: Float = 0.5) -> Bool
    end"#
);

impl RigzRandom for RandomModule {
    fn next_int(&self) -> i64 {
        rand::random()
    }

    fn next_float(&self) -> f64 {
        rand::random()
    }

    fn next_bool(&self, percent: f64) -> bool {
        rand::random_bool(percent)
    }
}
