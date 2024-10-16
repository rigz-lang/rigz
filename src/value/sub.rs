use crate::value::Value;
use crate::VMError;
use std::ops::Sub;

impl Sub for Value {
    type Output = Value;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) => Value::Error(v),
            (_, Value::Error(v)) => Value::Error(v),
            (Value::None, rhs) => -rhs,
            (lhs, Value::None) => lhs,
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(a | b),
            (Value::Bool(a), b) => Value::Bool(a | b.to_bool()),
            (b, Value::Bool(a)) => Value::Bool(a | b.to_bool()),
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            (Value::Number(a), Value::String(b)) => {
                let s = Value::String(b.clone());
                match s.to_number() {
                    None => VMError::UnsupportedOperation(format!("{} - {}", a, b)).to_value(),
                    Some(r) => Value::Number(a - r),
                }
            }
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
                result.retain(|v| *v != b);
                Value::List(result)
            }
            (Value::Map(a), Value::Map(b)) => {
                let mut result = a.clone();
                result.retain(|k, _| !b.contains_key(k));
                Value::Map(result)
            }
            (Value::Map(a), b) => {
                let mut result = a.clone();
                result.retain(|_, v| b != *v);
                Value::Map(result)
            }
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::define_value_tests;
    use crate::number::Number;
    use crate::value::Value;

    define_value_tests! {
        - {
            test_none_sub_none => (Value::None, Value::None, Value::None);
            test_none_bool_false_sub_none => (Value::Bool(false), Value::None, Value::Bool(false));
            test_bool_true_sub_none => (Value::Bool(true), Value::None, Value::Bool(true));
            test_none_bool_true_sub_true => (Value::None, Value::Bool(true), Value::Bool(false));
            test_false_bool_true_sub_true => (Value::Bool(false), Value::Bool(true), Value::Bool(true));
            test_false_0_sub_true => (Value::Bool(false), Value::Number(Number::zero()), Value::Bool(false));
            test_true_0_sub_true => (Value::Bool(true), Value::Number(Number::zero()), Value::Number(Number::one()));
            // sub more test cases here as needed
        }
    }
}
