use crate::{AsPrimitive, PrimitiveValue, VMError};
use std::ops::Sub;

impl Sub for &PrimitiveValue {
    type Output = PrimitiveValue;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (PrimitiveValue::Error(v), _) | (_, PrimitiveValue::Error(v)) => {
                PrimitiveValue::Error(v.clone())
            }
            (PrimitiveValue::Type(t), a) | (a, PrimitiveValue::Type(t)) => PrimitiveValue::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (-): {t} and {a}")),
            ),
            (PrimitiveValue::None, rhs) => -rhs,
            (lhs, PrimitiveValue::None) => lhs.clone(),
            (PrimitiveValue::Bool(a), PrimitiveValue::Bool(b)) => PrimitiveValue::Bool(a | b),
            (PrimitiveValue::Bool(a), b) => PrimitiveValue::Bool(a | b.to_bool()),
            (b, PrimitiveValue::Bool(a)) => PrimitiveValue::Bool(a | b.to_bool()),
            (PrimitiveValue::Number(a), PrimitiveValue::Number(b)) => PrimitiveValue::Number(a - b),
            (PrimitiveValue::Number(a), PrimitiveValue::String(b)) => match b.parse() {
                Err(_) => VMError::UnsupportedOperation(format!("{} - {}", a, b)).to_value(),
                Ok(r) => PrimitiveValue::Number(a / &r),
            },
            (PrimitiveValue::Number(a), PrimitiveValue::Range(r))
            | (PrimitiveValue::Range(r), PrimitiveValue::Number(a)) => match r - a {
                None => {
                    VMError::UnsupportedOperation(format!("Unable to subtract {a} from range {r}"))
                        .into()
                }
                Some(r) => PrimitiveValue::Range(r),
            },
            (PrimitiveValue::Range(a), PrimitiveValue::Range(b)) => match a - b {
                None => {
                    VMError::UnsupportedOperation(format!("Unable to subtract ranges: {a} - {b}"))
                        .into()
                }
                Some(r) => PrimitiveValue::Range(r),
            },
            (PrimitiveValue::String(a), PrimitiveValue::String(b)) => {
                let result = a.replace(b.as_str(), "");
                PrimitiveValue::String(result)
            }
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} - {rhs}")).into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::define_value_tests;

    define_value_tests! {
        - {
            test_none_sub_none => ((), ()) = ();
            test_none_bool_false_sub_none => (false, ()) = false;
            test_bool_true_sub_none => (true, ()) = true;
            test_none_bool_true_sub_true => ((), true) = false;
            test_false_bool_true_sub_true => (false, true) = true;
            test_false_0_sub_true => (false, 0) = false;
            test_true_0_sub_true => (true, 0) = 1;
        }
    }
}
