use crate::number::Number;
use crate::value::Value;
use crate::VMError;
use std::ops::Mul;

impl Mul for &Value {
    type Output = Value;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) | (_, Value::Error(v)) => Value::Error(v.clone()),
            (Value::Type(t), a) | (a, Value::Type(t)) => Value::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (*): {t} and {a}")),
            ),
            (Value::None, _) => Value::None,
            (_, Value::None) => Value::None,
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(a | b),
            (Value::Bool(a), b) => Value::Bool(a | b.to_bool()),
            (b, Value::Bool(a)) => Value::Bool(a | b.to_bool()),
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            (Value::Number(a), Value::String(b)) | (Value::String(b), Value::Number(a)) => {
                match b.parse() {
                    Err(_) => {
                        if a.is_negative() {
                            return VMError::RuntimeError(format!(
                                "Cannot multiply {} by negatives: {}",
                                b, a
                            ))
                            .to_value();
                        }

                        let s = match a {
                            Number::Int(_) => b.repeat(a.to_usize().unwrap()),
                            Number::Float(f) => {
                                let mut result = b.repeat(a.to_usize().unwrap());
                                result.push_str(&b[..(f.fract() * b.len() as f64) as usize]);
                                result
                            }
                        };
                        Value::String(s)
                    }
                    Ok(r) => Value::Number(a * &r),
                }
            }
            (Value::Number(a), Value::Range(r)) | (Value::Range(r), Value::Number(a)) => {
                match r * a {
                    None => VMError::UnsupportedOperation(format!("Unable to multiply {a} to {r}"))
                        .into(),
                    Some(r) => Value::Range(r),
                }
            }
            (Value::Range(a), Value::Range(b)) => match a * b {
                None => {
                    VMError::UnsupportedOperation(format!("Unable to multiply ranges: {a} * {b}"))
                        .into()
                }
                Some(r) => Value::Range(r),
            },
            (Value::String(a), Value::String(b)) => {
                &Value::List(vec![Value::String(a.clone())]) * &Value::String(b.clone())
            }
            (Value::Tuple(a), Value::Tuple(b)) => {
                Value::Tuple(a.iter().zip(b).map(|(a, b)| a * b).collect())
            }
            (Value::Tuple(a), b) => Value::Tuple(a.iter().map(|a| a * b).collect()),
            (b, Value::Tuple(a)) => Value::Tuple(a.iter().map(|a| b * a).collect()),
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} * {rhs}")).into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::define_value_tests;

    define_value_tests! {
        * {
            test_none_mul_none => ((), ()) = ();
            test_none_bool_false_mul_none => (false, ()) = false;
            test_bool_true_mul_none => (true, ()) = ();
            test_none_bool_true_mul_true => ((), true) = ();
            test_false_bool_true_mul_true => (false, true) = true;
            test_false_0_mul_true => (false, 0) = false;
            test_true_0_mul_true => (true, 0) = 1;
            test_str_f64_str => ("abc", 2.5) = "abcabca";
        }
    }
}
