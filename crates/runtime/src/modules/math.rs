use rigz_ast::*;
use rigz_ast_derive::derive_module;
use std::cell::RefCell;
use std::rc::Rc;

derive_module! {
    r#"import trait Math
    fn Number.log2 -> Number!
    fn Number.log10 -> Number!
    fn Number.logn(e: Number) -> Number!
    fn Number.pow(e: Number) -> Number!
    fn Number.sqrt -> Number!
    fn Number.sin -> Float
    fn Number.cos -> Float
    fn Number.tan -> Float
    fn Number.sinh -> Float
    fn Number.cosh -> Float
    fn Number.tanh -> Float
end"#
}

impl RigzMath for MathModule {
    fn number_log2(&self, this: Number) -> Result<Number, VMError> {
        this.log2()
    }

    fn number_log10(&self, this: Number) -> Result<Number, VMError> {
        this.log10()
    }

    fn number_logn(&self, this: Number, e: Number) -> Result<Number, VMError> {
        this.logn(e)
    }

    fn number_pow(&self, this: Number, e: Number) -> Result<Number, VMError> {
        this.pow(e)
    }

    fn number_sqrt(&self, this: Number) -> Result<Number, VMError> {
        this.sqrt()
    }

    fn number_sin(&self, this: Number) -> f64 {
        this.to_float().sin()
    }

    fn number_cos(&self, this: Number) -> f64 {
        this.to_float().cos()
    }

    fn number_tan(&self, this: Number) -> f64 {
        this.to_float().tan()
    }

    fn number_sinh(&self, this: Number) -> f64 {
        this.to_float().sinh()
    }

    fn number_cosh(&self, this: Number) -> f64 {
        this.to_float().cosh()
    }

    fn number_tanh(&self, this: Number) -> f64 {
        this.to_float().tanh()
    }
}
