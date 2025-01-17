use crate::{PrimitiveValue, VMError};
use std::ops::BitOr;

impl BitOr for &PrimitiveValue {
    type Output = PrimitiveValue;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (PrimitiveValue::Error(v), _) | (_, PrimitiveValue::Error(v)) => {
                PrimitiveValue::Error(v.clone())
            }
            (PrimitiveValue::Type(t), a) | (a, PrimitiveValue::Type(t)) => PrimitiveValue::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (|): {t} and {a}")),
            ),
            (PrimitiveValue::None, rhs) => rhs.clone(),
            (lhs, PrimitiveValue::None) => lhs.clone(),
            (PrimitiveValue::Bool(a), PrimitiveValue::Bool(b)) => PrimitiveValue::Bool(a | b),
            (PrimitiveValue::Bool(a), b) => PrimitiveValue::Bool(a | b.to_bool()),
            (b, PrimitiveValue::Bool(a)) => PrimitiveValue::Bool(a | b.to_bool()),
            (PrimitiveValue::Number(a), PrimitiveValue::Number(b)) => PrimitiveValue::Number(a | b),
            (PrimitiveValue::Number(a), PrimitiveValue::String(b)) => match b.parse() {
                Err(_) => VMError::UnsupportedOperation(format!("{} | {}", a, b)).to_value(),
                Ok(r) => PrimitiveValue::Number(a | &r),
            },
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} | {rhs}")).into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::define_value_tests;

    define_value_tests! {
        | {
            test_none_bitor_none => ((), ()) = ();
            test_none_bool_false_bitor_none => (false, ()) = false;
            test_bool_true_bitor_none => (true, ()) = true;
            test_none_bool_true_bitor_true => ((), true) = true;
            test_false_bool_true_bitor_true => (false, true) = true;
            test_false_0_bitor_true => (false, 0) = false;
            test_true_0_bitor_true => (true, 0) = 1;
        }
    }
}
