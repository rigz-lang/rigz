use rigz_ast::*;
use rigz_ast_derive::derive_module;
use rigz_core::*;
use std::cell::RefCell;
use std::rc::Rc;

derive_module! {
    r#"
import trait Number
    fn Number.ceil -> Number
    fn Number.round -> Number
    fn Number.trunc -> Number

    fn Number.min(other: Number) -> Number
    fn Number.max(other: Number) -> Number

    fn Number.log2 -> Number!
    fn Number.log10 -> Number!
    fn Number.logn(e: Number) -> Number!
    fn Number.pow(e: Number) -> Number!
    fn Number.sqrt -> Number!
    fn Number.abs -> Number
    fn Number.sin -> Float
    fn Number.cos -> Float
    fn Number.tan -> Float
    fn Number.sinh -> Float
    fn Number.cosh -> Float
    fn Number.tanh -> Float

    fn Number.to_bits -> [Bool]
    fn int_from_bits(raw: List) -> Int
    fn float_from_bits(raw: List) -> Float
end
"#
}

impl RigzNumber for NumberModule {
    fn number_ceil(&self, this: &Number) -> Number {
        match this {
            Number::Int(_) => *this,
            Number::Float(f) => (f.ceil() as i64).into(),
        }
    }

    fn number_round(&self, this: &Number) -> Number {
        match this {
            Number::Int(_) => *this,
            Number::Float(f) => (f.round() as i64).into(),
        }
    }

    fn number_trunc(&self, this: &Number) -> Number {
        match this {
            Number::Int(_) => *this,
            Number::Float(f) => (f.trunc() as i64).into(),
        }
    }

    fn number_min(&self, this: &Number, other: Number) -> Number {
        (*this).min(other)
    }

    fn number_max(&self, this: &Number, other: Number) -> Number {
        (*this).max(other)
    }

    fn number_log2(&self, this: &Number) -> Result<Number, VMError> {
        this.log2()
    }

    fn number_log10(&self, this: &Number) -> Result<Number, VMError> {
        this.log10()
    }

    fn number_logn(&self, this: &Number, e: Number) -> Result<Number, VMError> {
        this.logn(e)
    }

    fn number_pow(&self, this: &Number, e: Number) -> Result<Number, VMError> {
        this.pow(e)
    }

    fn number_sqrt(&self, this: &Number) -> Result<Number, VMError> {
        this.sqrt()
    }

    fn number_abs(&self, this: &Number) -> Number {
        match this {
            Number::Int(i) => i.abs().into(),
            Number::Float(f) => f.abs().into(),
        }
    }

    fn number_sin(&self, this: &Number) -> f64 {
        this.to_float().sin()
    }

    fn number_cos(&self, this: &Number) -> f64 {
        this.to_float().cos()
    }

    fn number_tan(&self, this: &Number) -> f64 {
        this.to_float().tan()
    }

    fn number_sinh(&self, this: &Number) -> f64 {
        this.to_float().sinh()
    }

    fn number_cosh(&self, this: &Number) -> f64 {
        this.to_float().cosh()
    }

    fn number_tanh(&self, this: &Number) -> f64 {
        this.to_float().tanh()
    }

    fn number_to_bits(&self, this: &Number) -> Vec<bool> {
        let bits = this.to_bits();
        let start = bits.leading_zeros();
        let bits = bits.reverse_bits();
        (start..64)
            .map(|index| {
                let mask = 1 << index;
                bits & mask == mask
            })
            .collect()
    }

    fn int_from_bits(&self, raw: Vec<Rc<RefCell<ObjectValue>>>) -> i64 {
        raw.into_iter()
            .rev()
            .enumerate()
            .fold(0, |res, (index, next)| {
                res | ((next.borrow().to_bool() as i64) << index as i64)
            })
    }

    fn float_from_bits(&self, raw: Vec<Rc<RefCell<ObjectValue>>>) -> f64 {
        let raw = raw
            .into_iter()
            .rev()
            .enumerate()
            .fold(0, |res, (index, next)| {
                res | ((next.borrow().to_bool() as u64) << index as u64)
            });
        f64::from_bits(raw)
    }
}
