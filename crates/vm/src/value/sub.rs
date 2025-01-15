use crate::value::Value;
use crate::VMError;
use std::ops::Sub;

impl Sub for &Value {
    type Output = Value;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) | (_, Value::Error(v)) => Value::Error(v.clone()),
            (Value::Type(t), a) | (a, Value::Type(t)) => Value::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (-): {t} and {a}")),
            ),
            (Value::None, rhs) => -rhs,
            (lhs, Value::None) => lhs.clone(),
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(a | b),
            (Value::Bool(a), b) => Value::Bool(a | b.to_bool()),
            (b, Value::Bool(a)) => Value::Bool(a | b.to_bool()),
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            (Value::Number(a), Value::String(b)) => {
                let s = Value::String(b.clone());
                match s.to_number() {
                    Err(_) => VMError::UnsupportedOperation(format!("{} - {}", a, b)).to_value(),
                    Ok(r) => Value::Number(a / &r),
                }
            }
            (Value::Number(a), Value::Range(r)) | (Value::Range(r), Value::Number(a)) => {
                match r - a {
                    None => VMError::UnsupportedOperation(format!(
                        "Unable to subtract {a} from range {r}"
                    ))
                    .into(),
                    Some(r) => Value::Range(r),
                }
            }
            (Value::Range(a), Value::Range(b)) => match a - b {
                None => {
                    VMError::UnsupportedOperation(format!("Unable to subtract ranges: {a} - {b}"))
                        .into()
                }
                Some(r) => Value::Range(r),
            },
            (Value::String(a), Value::String(b)) => {
                let result = a.replace(b.as_str(), "");
                Value::String(result)
            }
            (Value::List(a), Value::List(b)) => {
                let mut result = a.clone();
                result.retain(|v| !b.contains(v));
                Value::List(result)
            }
            (Value::List(a), b) => {
                let mut result = a.clone();
                result.retain(|v| v != b);
                Value::List(result)
            }
            (Value::Map(a), Value::Map(b)) => {
                let mut result = a.clone();
                result.retain(|k, _| !b.contains_key(k));
                Value::Map(result)
            }
            (Value::Map(a), b) => {
                let mut result = a.clone();
                result.retain(|_, v| b != v);
                Value::Map(result)
            }
            (Value::Tuple(a), Value::Tuple(b)) => {
                Value::Tuple(a.iter().zip(b).map(|(a, b)| a - b).collect())
            }
            (Value::Tuple(a), b) => Value::Tuple(a.iter().map(|a| a - b).collect()),
            (b, Value::Tuple(a)) => Value::Tuple(a.iter().map(|a| b - a).collect()),
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
