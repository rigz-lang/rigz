use crate::{PrimitiveValue, ToBool, VMError};
use log::warn;
use std::ops::{Rem, RemAssign};

impl Rem for &PrimitiveValue {
    type Output = PrimitiveValue;

    #[inline]
    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (PrimitiveValue::Error(v), _) | (_, PrimitiveValue::Error(v)) => {
                PrimitiveValue::Error(v.clone())
            }
            (PrimitiveValue::Type(t), a) | (a, PrimitiveValue::Type(t)) => PrimitiveValue::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (%): {t} and {a}")),
            ),
            (PrimitiveValue::None, _) => PrimitiveValue::None,
            (lhs, PrimitiveValue::None) => lhs.clone(),
            (PrimitiveValue::Bool(a), PrimitiveValue::Bool(b)) => PrimitiveValue::Bool(a | b),
            (PrimitiveValue::Bool(a), b) => PrimitiveValue::Bool(a | b.to_bool()),
            (b, PrimitiveValue::Bool(a)) => PrimitiveValue::Bool(a | b.to_bool()),
            (PrimitiveValue::Number(a), PrimitiveValue::Number(b)) => PrimitiveValue::Number(a % b),
            (PrimitiveValue::Number(a), PrimitiveValue::String(b)) => match b.parse() {
                Err(_) => VMError::UnsupportedOperation(format!("{} % {}", a, b)).into(),
                Ok(r) => PrimitiveValue::Number(a % &r),
            },
            (a, b) => {
                warn!("{a} % {b} not implemented, defaulting to a - b");
                a - b
            }
        }
    }
}

impl RemAssign<&PrimitiveValue> for PrimitiveValue {
    fn rem_assign(&mut self, rhs: &Self) {
        *self = self.rem(rhs);
    }
}

#[cfg(test)]
mod tests {
    use crate::define_value_tests;

    define_value_tests! {
        % {
            test_none_rem_none => ((), ()) = ();
            test_none_bool_false_rem_none => (false, ()) = false;
            test_bool_true_rem_none => (true, ()) = true;
            test_none_bool_true_rem_true => ((), true) = ();
            test_false_bool_true_rem_true => (false, true) = true;
            test_false_0_rem_true => (false, 0) = false;
            test_true_0_rem_true => (true, 0) = 1;
        }
    }
}
