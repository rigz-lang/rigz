use crate::value::Value;
use crate::VMError;
use std::ops::Div;

impl Div for &Value {
    type Output = Value;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) | (_, Value::Error(v)) => Value::Error(v.clone()),
            (Value::Type(t), a) | (a, Value::Type(t)) => Value::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (/): {t} and {a}")),
            ),
            (Value::None, _) => Value::None,
            (lhs, Value::None) => Value::Error(VMError::RuntimeError(format!(
                "Cannot divide {} by 0/none",
                lhs
            ))),
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(a | b),
            (Value::Bool(a), b) => Value::Bool(a | b.to_bool()),
            (b, Value::Bool(a)) => Value::Bool(a | b.to_bool()),
            (Value::Number(a), Value::Number(b)) => {
                if b.is_zero() {
                    return Value::Error(VMError::RuntimeError(format!(
                        "Cannot divide {} by 0/none",
                        a
                    )));
                }

                Value::Number(a / b)
            }
            (Value::Number(a), Value::String(b)) => {
                let s = Value::String(b.clone());
                match s.to_number() {
                    Err(_) => VMError::UnsupportedOperation(format!("{} / {}", a, b)).to_value(),
                    Ok(r) => Value::Number(a / &r),
                }
            }
            (Value::String(a), Value::String(b)) => {
                let result = a.split(b.as_str());
                Value::List(result.map(|s| Value::String(s.to_string())).collect())
            }
            (Value::String(a), b) => {
                let b = b.to_string();
                let result = a.split(b.as_str());
                Value::List(result.map(|s| Value::String(s.to_string())).collect())
            }
            (Value::Tuple(a), Value::Tuple(b)) => {
                Value::Tuple(a.iter().zip(b).map(|(a, b)| a / b).collect())
            }
            (Value::Tuple(a), b) => Value::Tuple(a.iter().map(|a| a / b).collect()),
            (b, Value::Tuple(a)) => Value::Tuple(a.iter().map(|a| b / a).collect()),
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} / {rhs}")).into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{define_value_tests, VMError};

    define_value_tests! {
        / {
            test_none_div_none => ((), ()) = ();
            test_none_bool_false_div_none => (false, ()) = VMError::RuntimeError("Cannot divide false by 0/none".to_string());
            test_bool_true_div_none => (true, ()) = VMError::RuntimeError("Cannot divide true by 0/none".to_string());
            test_none_bool_true_div_true => ((), true) = ();
            test_false_bool_true_div_true => (false, true) = true;
            test_false_0_div_true => (false, 0) = false;
            test_true_0_div_true => (true, 0) = 1;
        }
    }
}
