use rigz_ast::*;
use rigz_ast_derive::derive_module;
use std::cell::RefCell;
use std::rc::Rc;

derive_module!(
    r#"
import trait Number
    fn Number.ceil -> Number
    fn Number.round -> Number
    fn Number.trunc -> Number

    fn Number.to_bits -> List
    fn int_from_bits(raw: List) -> Int
    fn float_from_bits(raw: List) -> Float
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

    fn number_to_bits(&self, this: Number) -> Vec<Value> {
        let bits = this.to_bits();
        let start = bits.leading_zeros();
        let bits = bits.reverse_bits();
        (start..64)
            .map(|index| {
                let mask = 1 << index;
                (bits & mask == mask).into()
            })
            .collect()
    }

    fn int_from_bits(&self, raw: Vec<Value>) -> i64 {
        raw.into_iter()
            .rev()
            .enumerate()
            .fold(0, |res, (index, next)| {
                res | ((next.to_bool() as i64) << index as i64)
            })
    }

    fn float_from_bits(&self, raw: Vec<Value>) -> f64 {
        let raw = raw
            .into_iter()
            .rev()
            .enumerate()
            .fold(0, |res, (index, next)| {
                res | ((next.to_bool() as u64) << index as u64)
            });
        f64::from_bits(raw)
    }
}
