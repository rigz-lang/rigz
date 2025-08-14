use crate::{PrimitiveValue, ToBool, VMError};
use std::ops::BitAnd;

impl BitAnd for &PrimitiveValue {
    type Output = PrimitiveValue;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (PrimitiveValue::Error(v), _) | (_, PrimitiveValue::Error(v)) => {
                PrimitiveValue::Error(v.clone())
            }
            (PrimitiveValue::Type(t), a) | (a, PrimitiveValue::Type(t)) => PrimitiveValue::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (&): {t} and {a}")),
            ),
            (PrimitiveValue::None, _) => PrimitiveValue::None,
            (_, PrimitiveValue::None) => PrimitiveValue::None,
            (PrimitiveValue::Bool(a), PrimitiveValue::Bool(b)) => PrimitiveValue::Bool(a & b),
            (PrimitiveValue::Bool(a), b) => PrimitiveValue::Bool(a & b.to_bool()),
            (b, PrimitiveValue::Bool(a)) => PrimitiveValue::Bool(a & b.to_bool()),
            (PrimitiveValue::Number(a), PrimitiveValue::Number(b)) => PrimitiveValue::Number(a & b),
            (PrimitiveValue::Number(a), PrimitiveValue::String(b)) => match b.parse() {
                Err(_) => VMError::UnsupportedOperation(format!("{} & {}", a, b)).into(),
                Ok(r) => PrimitiveValue::Number(a & &r),
            },
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} & {rhs}")).into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::define_value_tests;

    define_value_tests! {
        & {
            test_none_bitand_none => ((), ()) = ();
            test_none_bool_false_bitand_none => (false, ()) = false;
            test_bool_true_bitand_none => (true, ()) = ();
            test_none_bool_true_bitand_true => ((), true) = ();
            test_false_bool_true_bitand_true => (false, true) = false;
            test_false_0_bitand_true => (false, 0) = false;
            test_true_0_bitand_true => (true, 0) = false;
        }
    }
}
