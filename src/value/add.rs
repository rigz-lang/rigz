use crate::value::Value;
use crate::VMError;
use std::ops::Add;

impl Add for Value {
    type Output = Value;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) | (_, Value::Error(v)) => Value::Error(v),
            (Value::None, v) | (v, Value::None) => v,
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(a | b),
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::Number(a), Value::String(b)) | (Value::String(b), Value::Number(a)) => {
                let s = Value::String(b.clone());
                match s.to_number() {
                    None => {
                        let mut res = a.to_string();
                        res.push_str(b.as_str());
                        Value::String(res)
                    }
                    Some(r) => Value::Number(a + r),
                }
            }
            (Value::Number(a), Value::Range(r)) | (Value::Range(r), Value::Number(a)) => {
                match r + a {
                    None => {
                        VMError::UnsupportedOperation(format!("Unable to perform add {a} to range"))
                            .to_value()
                    }
                    Some(r) => Value::Range(r),
                }
            }
            (Value::Range(a), Value::Range(b)) => match a + b {
                None => VMError::UnsupportedOperation("Unable to perform add ranges".to_string())
                    .to_value(),
                Some(r) => Value::Range(r),
            },
            (Value::Range(a), Value::String(b)) | (Value::String(b), Value::Range(a)) => {
                VMError::UnsupportedOperation(format!("Cannot perform {a} + {b}")).to_value()
            }
            (Value::String(a), Value::String(b)) => {
                let mut result = a.clone();
                result.push_str(b.as_str());
                Value::String(result)
            }
            (Value::List(a), Value::List(b)) => {
                let mut result = a.clone();
                result.extend(b);
                Value::List(result)
            }
            (Value::List(a), b) | (b, Value::List(a)) => {
                let mut result = a.clone();
                result.push(b);
                Value::List(result)
            }
            (Value::Map(a), Value::Map(b)) => {
                let mut result = a.clone();
                result.extend(b);
                Value::Map(result)
            }
            (Value::Map(a), b) | (b, Value::Map(a)) => {
                let mut result = a.clone();
                result.insert(b.clone(), b);
                Value::Map(result)
            }
            (Value::Bool(a), b) | (b, Value::Bool(a)) => Value::Bool(a | b.to_bool()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::define_value_tests;
    use crate::number::Number;
    use crate::value::Value;

    define_value_tests! {
        + {
            test_none_add_none => (Value::None, Value::None, Value::None);
            test_none_bool_false_add_none => (Value::Bool(false), Value::None, Value::Bool(false));
            test_bool_true_add_none => (Value::Bool(true), Value::None, Value::Bool(true));
            test_none_bool_true_add_true => (Value::None, Value::Bool(true), Value::Bool(true));
            test_false_bool_true_add_true => (Value::Bool(false), Value::Bool(true), Value::Bool(true));
            test_false_0_add_true => (Value::Bool(false), Value::Number(Number::Int(0)), Value::Bool(false));
            test_true_0_add_true => (Value::Bool(true), Value::Number(Number::Int(0)), Value::Number(Number::Int(1)));
            // Add more test cases here as needed
        }
    }
}
