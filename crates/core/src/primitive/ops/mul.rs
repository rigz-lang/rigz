use crate::{AsPrimitive, Number, PrimitiveValue, VMError};
use std::ops::Mul;

impl Mul for &PrimitiveValue {
    type Output = PrimitiveValue;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (PrimitiveValue::Error(v), _) | (_, PrimitiveValue::Error(v)) => {
                PrimitiveValue::Error(v.clone())
            }
            (PrimitiveValue::Type(t), a) | (a, PrimitiveValue::Type(t)) => PrimitiveValue::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (*): {t} and {a}")),
            ),
            (PrimitiveValue::None, _) => PrimitiveValue::None,
            (_, PrimitiveValue::None) => PrimitiveValue::None,
            (PrimitiveValue::Bool(a), PrimitiveValue::Bool(b)) => PrimitiveValue::Bool(a | b),
            (PrimitiveValue::Bool(a), b) => PrimitiveValue::Bool(a | b.to_bool()),
            (b, PrimitiveValue::Bool(a)) => PrimitiveValue::Bool(a | b.to_bool()),
            (PrimitiveValue::Number(a), PrimitiveValue::Number(b)) => PrimitiveValue::Number(a * b),
            (PrimitiveValue::Number(a), PrimitiveValue::String(b))
            | (PrimitiveValue::String(b), PrimitiveValue::Number(a)) => match b.parse() {
                Err(_) => {
                    if a.is_negative() {
                        return VMError::runtime(format!(
                            "Cannot multiply {} by negatives: {}",
                            b, a
                        ))
                        .to_value();
                    }

                    let s = match a {
                        Number::Int(_) => b.repeat(a.to_usize().unwrap()),
                        Number::Float(f) => {
                            let mut result = b.repeat(a.to_usize().unwrap());
                            result.push_str(&b[..(f.fract() * b.len() as f64) as usize]);
                            result
                        }
                    };
                    PrimitiveValue::String(s)
                }
                Ok(r) => PrimitiveValue::Number(a * &r),
            },
            (PrimitiveValue::Number(a), PrimitiveValue::Range(r))
            | (PrimitiveValue::Range(r), PrimitiveValue::Number(a)) => match r * a {
                None => {
                    VMError::UnsupportedOperation(format!("Unable to multiply {a} to {r}")).into()
                }
                Some(r) => PrimitiveValue::Range(r),
            },
            (PrimitiveValue::Range(a), PrimitiveValue::Range(b)) => match a * b {
                None => {
                    VMError::UnsupportedOperation(format!("Unable to multiply ranges: {a} * {b}"))
                        .into()
                }
                Some(r) => PrimitiveValue::Range(r),
            },
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} * {rhs}")).into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::define_value_tests;

    define_value_tests! {
        * {
            test_none_mul_none => ((), ()) = ();
            test_none_bool_false_mul_none => (false, ()) = false;
            test_bool_true_mul_none => (true, ()) = ();
            test_none_bool_true_mul_true => ((), true) = ();
            test_false_bool_true_mul_true => (false, true) = true;
            test_false_0_mul_true => (false, 0) = false;
            test_true_0_mul_true => (true, 0) = 1;
            test_str_f64_str => ("abc", 2.5) = "abcabca";
        }
    }
}
