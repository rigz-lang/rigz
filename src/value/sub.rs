use std::ops::{Sub};
use crate::value::Value;

impl <'vm> Sub for Value<'vm> {
    type Output = Value<'vm>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) => Value::Error(v),
            (_, Value::Error(v)) => Value::Error(v),
            (Value::None, rhs) => -rhs,
            (lhs, Value::None) => lhs,
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(a | b),
            (Value::Bool(a), b) => Value::Bool(a | b.to_bool()),
            (b, Value::Bool(a)) => Value::Bool(a | b.to_bool()),
            (Value::Number(a), Value::Number(b)) => {
                match a - b {
                    Ok(n) => Value::Number(n),
                    Err(e) => Value::Error(e)
                }
            },
            (Value::String(a), Value::String(b)) => {
                let result = a.replacen(b.as_str(), "", 1);
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
            _ => todo!()
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
            test_false_0_sub_true => (Value::Bool(false), Value::Number(Number::UInt(0)), Value::Bool(false));
            test_true_0_sub_true => (Value::Bool(true), Value::Number(Number::UInt(0)), Value::Number(Number::UInt(1)));
            // sub more test cases here as needed
        }
    }
}