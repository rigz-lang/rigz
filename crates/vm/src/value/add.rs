use crate::value::Value;
use crate::VMError;
use std::ops::Add;

impl Add for &Value {
    type Output = Value;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) | (_, Value::Error(v)) => v.into(),
            (Value::Type(t), a) | (a, Value::Type(t)) => Value::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (+): {t} and {a}")),
            ),
            (Value::None, v) | (v, Value::None) => v.clone(),
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(a | b),
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::Number(a), Value::String(b)) => {
                let s = Value::String(b.clone());
                match s.to_number() {
                    Err(_) => {
                        let mut res = a.to_string();
                        res.push_str(b.as_str());
                        Value::String(res)
                    }
                    Ok(r) => Value::Number(a + &r),
                }
            }
            (Value::String(a), Value::Number(b)) => {
                let s = Value::String(a.clone());
                match s.to_number() {
                    Err(_) => {
                        let mut res = a.to_string();
                        res.push_str(b.to_string().as_str());
                        Value::String(res)
                    }
                    Ok(r) => Value::Number(b + &r),
                }
            }
            (Value::Number(a), Value::Range(r)) | (Value::Range(r), Value::Number(a)) => {
                match r + a {
                    None => {
                        VMError::UnsupportedOperation(format!("Unable to perform add {a} to range"))
                            .into()
                    }
                    Some(r) => Value::Range(r),
                }
            }
            (Value::Range(a), Value::Range(b)) => match a + b {
                None => {
                    VMError::UnsupportedOperation("Unable to perform add ranges".to_string()).into()
                }
                Some(r) => Value::Range(r),
            },
            (Value::Range(a), Value::String(b)) | (Value::String(b), Value::Range(a)) => {
                VMError::UnsupportedOperation(format!("Cannot perform {a} + {b}")).into()
            }
            (Value::String(a), Value::String(b)) => {
                let mut result = a.clone();
                result.push_str(b.as_str());
                Value::String(result)
            }
            (Value::Tuple(a), Value::Tuple(b)) => {
                Value::Tuple(a.iter().zip(b).map(|(a, b)| a + b).collect())
            }
            (Value::Tuple(a), b) => Value::Tuple(a.iter().map(|a| a + b).collect()),
            (b, Value::Tuple(a)) => Value::Tuple(a.iter().map(|a| b + a).collect()),
            (Value::List(a), Value::List(b)) => {
                let mut result = a.clone();
                result.extend(b.clone());
                Value::List(result)
            }
            (Value::List(a), b) => {
                let mut result = a.clone();
                result.push(b.clone());
                Value::List(result)
            }
            (b, Value::List(a)) => {
                let mut result = Vec::with_capacity(a.len() + 1);
                result.push(b.clone());
                result.extend(a.clone());
                Value::List(result)
            }
            (Value::Map(a), Value::Map(b)) => {
                let mut result = a.clone();
                result.extend(b.clone());
                Value::Map(result)
            }
            (Value::Map(a), b) | (b, Value::Map(a)) => {
                let mut result = a.clone();
                result.insert(b.clone(), b.clone());
                Value::Map(result)
            }
            // todo should "a" + true = "atrue" or true
            (Value::Bool(a), b) | (b, Value::Bool(a)) => Value::Bool(a | b.to_bool()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::define_value_tests;

    define_value_tests! {
        + {
            test_none_add_none => ((), ()) = ();
            test_none_bool_false_add_none => (false, ()) = false;
            test_bool_true_add_none => (true, ()) = true;
            test_none_bool_true_add_true => ((), true) = true;
            test_false_bool_true_add_true => (false, true) = true;
            test_false_0_add_true => (false, 0) = false;
            test_true_0_add_true => (true, 0) = 1;
            test_str_1_add_num => ("1", 6) = 7;
            test_str_abc_add_num => ("abc", 6) = "abc6";
            test_num_add_abc => (6, "abc") = "6abc";
        }
    }
}
