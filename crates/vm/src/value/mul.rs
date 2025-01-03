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
            (Value::Number(a), Value::String(b)) => {
                let s = Value::String(b.clone());
                match s.to_number() {
                    Err(_) => VMError::UnsupportedOperation(format!("{} * {}", a, b)).into(),
                    Ok(r) => Value::Number(a * &r),
                }
            }
            (Value::String(a), Value::Number(n)) => {
                if n.is_negative() {
                    return VMError::RuntimeError(format!(
                        "Cannot multiply {} by negatives: {}",
                        a, n
                    ))
                    .to_value();
                }

                let s = match n {
                    Number::Int(_) => a.repeat(n.to_usize().unwrap()),
                    Number::Float(f) => {
                        let mut result = a.repeat(n.to_usize().unwrap());
                        result.push_str(&a[..(f.fract() * a.len() as f64) as usize]);
                        result
                    }
                };
                Value::String(s)
            }
            (Value::String(a), Value::String(b)) => {
                &Value::List(vec![Value::String(a.clone())]) * &Value::String(b.clone())
            }
            (Value::Tuple(a), Value::Tuple(b)) => {
                Value::Tuple(a.iter().zip(b).map(|(a, b)| a * b).collect())
            }
            (Value::Tuple(a), b) => Value::Tuple(a.iter().map(|a| a * b).collect()),
            (b, Value::Tuple(a)) => Value::Tuple(a.iter().map(|a| b * a).collect()),
            // (Value::String(a), b) => {
            //     let mut result = a.clone();
            //     result.push_str(b.to_string().as_str());
            //     Value::String(result)
            // }
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
            (lhs, rhs) => todo!("{lhs} * {rhs}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::define_value_tests;
    use crate::number::Number;
    use crate::value::Value;

    define_value_tests! {
        * {
            test_none_mul_none => (Value::None, Value::None, Value::None);
            test_none_bool_false_mul_none => (Value::Bool(false), Value::None, Value::Bool(false));
            test_bool_true_mul_none => (Value::Bool(true), Value::None, Value::None);
            test_none_bool_true_mul_true => (Value::None, Value::Bool(true), Value::None);
            test_false_bool_true_mul_true => (Value::Bool(false), Value::Bool(true), Value::Bool(true));
            test_false_0_mul_true => (Value::Bool(false), Value::Number(Number::Int(0)), Value::Bool(false));
            test_true_0_mul_true => (Value::Bool(true), Value::Number(Number::Int(0)), Value::Number(Number::Int(1)));
            test_str_f64_str => (Value::String("abc".to_string()), Value::Number(Number::Float(2.5)), Value::String("abcabca".to_string()));
            // todo add more test cases
        }
    }
}
