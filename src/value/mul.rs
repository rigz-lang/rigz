use std::ops::{Mul};
use crate::number::Number;
use crate::value::Value;
use crate::VMError;

impl <'vm> Mul for Value<'vm> {
    type Output = Value<'vm>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) => Value::Error(v),
            (_, Value::Error(v)) => Value::Error(v),
            (Value::None, _) => Value::None,
            (_, Value::None) => Value::None,
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(a | b),
            (Value::Bool(a), b) => Value::Bool(a | b.to_bool()),
            (b, Value::Bool(a)) => Value::Bool(a | b.to_bool()),
            (Value::Number(a), Value::Number(b)) => {
                match a * b {
                    Ok(n) => Value::Number(n),
                    Err(e) => Value::Error(e)
                }
            },
            (Value::String(a), Value::Number(n)) => {
                if n.is_negative() {
                    return Value::Error(VMError::RuntimeError(format!("Cannot multiply {} by negatives: {}", a, n.to_string())))
                }

                let s = match n {
                    Number::Int(_) | Number::UInt(_) => a.repeat(n.to_usize().unwrap()),
                    Number::Float(f) => {
                        let mut result = a.repeat(n.to_usize().unwrap());
                        result.push_str(&a[..(f.fract() * a.len() as f64) as usize]);
                        result
                    }
                };
                Value::String(s)
            }
            (Value::String(a), Value::String(b)) => {
                Value::List(vec![Value::String(a)]) * Value::String(b)
            }
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
            _ => todo!()
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
        * {
            test_none_mul_none => (Value::None, Value::None, Value::None);
            test_none_bool_false_mul_none => (Value::Bool(false), Value::None, Value::Bool(false));
            test_bool_true_mul_none => (Value::Bool(true), Value::None, Value::None);
            test_none_bool_true_mul_true => (Value::None, Value::Bool(true), Value::None);
            test_false_bool_true_mul_true => (Value::Bool(false), Value::Bool(true), Value::Bool(true));
            test_false_0_mul_true => (Value::Bool(false), Value::Number(Number::UInt(0)), Value::Bool(false));
            test_true_0_mul_true => (Value::Bool(true), Value::Number(Number::UInt(0)), Value::Number(Number::UInt(1)));
            test_str_f64_str => (Value::String("abc".to_string()), Value::Number(Number::Float(2.5)), Value::String("abcabca".to_string()));
            // mul more test cases here as needed
        }
    }
}