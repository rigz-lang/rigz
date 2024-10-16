use crate::value::Value;
use crate::VMError;
use std::ops::Div;

impl Div for Value {
    type Output = Value;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) => Value::Error(v),
            (_, Value::Error(v)) => Value::Error(v),
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
                    None => VMError::UnsupportedOperation(format!("{} / {}", a, b)).to_value(),
                    Some(r) => Value::Number(a / r),
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
            // (Value::List(a), Value::List(b)) => {
            //     let mut result = a.clone();
            //     result.extend(b);
            //     Value::List(result)
            // }
            // (Value::List(a), b) => {
            //     let mut result = a.clone();
            //     result.push(b);
            //     Value::List(result)
            // }
            // (Value::Map(a), Value::Map(b)) => {
            //     let mut result = a.clone();
            //     result.extend(b);
            //     Value::Map(result)
            // }
            // (Value::Map(a), b) => {
            //     let mut result = a.clone();
            //     result.insert(b.clone(), b);
            //     Value::Map(result)
            // }
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::define_value_tests;
    use crate::number::Number;
    use crate::value::Value;
    use crate::VMError::RuntimeError;

    define_value_tests! {
        / {
            test_none_div_none => (Value::None, Value::None, Value::None);
            test_none_bool_false_div_none => (Value::Bool(false), Value::None, Value::Error(RuntimeError("Cannot divide false by 0/none".to_string())));
            test_bool_true_div_none => (Value::Bool(true), Value::None, Value::Error(RuntimeError("Cannot divide true by 0/none".to_string())));
            test_none_bool_true_div_true => (Value::None, Value::Bool(true), Value::None);
            test_false_bool_true_div_true => (Value::Bool(false), Value::Bool(true), Value::Bool(true));
            test_false_0_div_true => (Value::Bool(false), Value::Number(Number::zero()), Value::Bool(false));
            test_true_0_div_true => (Value::Bool(true), Value::Number(Number::zero()), Value::Number(Number::one()));
            // div more test cases here as needed
        }
    }
}
