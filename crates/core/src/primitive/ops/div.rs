use crate::{PrimitiveValue, ToBool, VMError};
use std::ops::{Div, DivAssign};

impl Div for &PrimitiveValue {
    type Output = PrimitiveValue;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (PrimitiveValue::Error(v), _) | (_, PrimitiveValue::Error(v)) => {
                PrimitiveValue::Error(v.clone())
            }
            (PrimitiveValue::Type(t), a) | (a, PrimitiveValue::Type(t)) => PrimitiveValue::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (/): {t} and {a}")),
            ),
            (PrimitiveValue::None, _) => PrimitiveValue::None,
            (lhs, PrimitiveValue::None) => {
                PrimitiveValue::Error(VMError::runtime(format!("Cannot divide {} by 0/none", lhs)))
            }
            (PrimitiveValue::Bool(a), PrimitiveValue::Bool(b)) => PrimitiveValue::Bool(a | b),
            (PrimitiveValue::Bool(a), b) => PrimitiveValue::Bool(a | b.to_bool()),
            (b, PrimitiveValue::Bool(a)) => PrimitiveValue::Bool(a | b.to_bool()),
            (PrimitiveValue::Number(a), PrimitiveValue::Number(b)) => {
                if b.is_zero() {
                    return PrimitiveValue::Error(VMError::runtime(format!(
                        "Cannot divide {} by 0/none",
                        a
                    )));
                }

                PrimitiveValue::Number(a / b)
            }
            (PrimitiveValue::Number(a), PrimitiveValue::String(b)) => match b.parse() {
                Err(_) => VMError::UnsupportedOperation(format!("{} / {}", a, b)).to_value(),
                Ok(r) => PrimitiveValue::Number(a / &r),
            },
            (PrimitiveValue::Number(a), PrimitiveValue::Range(r))
            | (PrimitiveValue::Range(r), PrimitiveValue::Number(a)) => match r / a {
                None => VMError::UnsupportedOperation(format!("Unable to div {a} from {r}")).into(),
                Some(r) => PrimitiveValue::Range(r),
            },
            (PrimitiveValue::Range(a), PrimitiveValue::Range(b)) => match a / b {
                None => {
                    VMError::UnsupportedOperation(format!("Unable to divide ranges: {a} / {b}"))
                        .into()
                }
                Some(r) => PrimitiveValue::Range(r),
            },
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} / {rhs}")).into()
            }
        }
    }
}

impl DivAssign<&PrimitiveValue> for PrimitiveValue {
    fn div_assign(&mut self, rhs: &Self) {
        *self = self.div(rhs);
    }
}

#[cfg(test)]
mod tests {
    use crate::{define_value_tests, VMError};

    define_value_tests! {
        / {
            test_none_div_none => ((), ()) = ();
            test_none_bool_false_div_none => (false, ()) = VMError::runtime("Cannot divide false by 0/none".to_string());
            test_bool_true_div_none => (true, ()) = VMError::runtime("Cannot divide true by 0/none".to_string());
            test_none_bool_true_div_true => ((), true) = ();
            test_false_bool_true_div_true => (false, true) = true;
            test_false_0_div_true => (false, 0) = false;
            test_true_0_div_true => (true, 0) = 1;
        }
    }
}
