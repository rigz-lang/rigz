use crate::value::Value;
use crate::VMError;
use std::ops::BitOr;

impl BitOr for &Value {
    type Output = Value;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) | (_, Value::Error(v)) => Value::Error(v.clone()),
            (Value::Type(t), a) | (a, Value::Type(t)) => Value::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (|): {t} and {a}")),
            ),
            (Value::None, rhs) => rhs.clone(),
            (lhs, Value::None) => lhs.clone(),
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(a | b),
            (Value::Bool(a), b) => Value::Bool(a | b.to_bool()),
            (b, Value::Bool(a)) => Value::Bool(a | b.to_bool()),
            (Value::Number(a), Value::Number(b)) => Value::Number(a | b),
            (Value::Number(a), Value::String(b)) => {
                match b.parse() {
                    Err(_) => VMError::UnsupportedOperation(format!("{} | {}", a, b)).to_value(),
                    Ok(r) => Value::Number(a | &r),
                }
            }
            (Value::Tuple(a), Value::Tuple(b)) => {
                Value::Tuple(a.iter().zip(b).map(|(a, b)| a | b).collect())
            }
            (Value::Tuple(a), b) => Value::Tuple(a.iter().map(|a| a | b).collect()),
            (b, Value::Tuple(a)) => Value::Tuple(a.iter().map(|a| b | a).collect()),
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
