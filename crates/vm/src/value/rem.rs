use crate::value::Value;
use crate::VMError;
use log::warn;
use std::ops::Rem;

impl Rem for &Value {
    type Output = Value;

    #[inline]
    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) | (_, Value::Error(v)) => Value::Error(v.clone()),
            (Value::Type(t), a) | (a, Value::Type(t)) => Value::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (%): {t} and {a}")),
            ),
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs.clone(),
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(a | b),
            (Value::Bool(a), b) => Value::Bool(a | b.to_bool()),
            (b, Value::Bool(a)) => Value::Bool(a | b.to_bool()),
            (Value::Number(a), Value::Number(b)) => Value::Number(a % b),
            (Value::Number(a), Value::String(b)) => {
                let s = Value::String(b.clone());
                match s.to_number() {
                    Err(_) => VMError::UnsupportedOperation(format!("{} % {}", a, b)).into(),
                    Ok(r) => Value::Number(a % &r),
                }
            }
            (Value::Tuple(a), Value::Tuple(b)) => {
                Value::Tuple(a.iter().zip(b).map(|(a, b)| a % b).collect())
            }
            (Value::Tuple(a), b) => Value::Tuple(a.iter().map(|a| a % b).collect()),
            (b, Value::Tuple(a)) => Value::Tuple(a.iter().map(|a| b % a).collect()),
            (a, b) => {
                warn!("{a} % {b} not implemented, defaulting to a - b");
                a - b
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{define_value_tests};

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
