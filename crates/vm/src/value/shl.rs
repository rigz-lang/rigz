use crate::number::Number;
use crate::value::Value;
use crate::VMError;
use std::ops::Shl;

impl Shl for &Value {
    type Output = Value;

    #[inline]
    fn shl(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) | (_, Value::Error(v)) => Value::Error(v.clone()),
            (Value::Type(t), a) | (a, Value::Type(t)) => Value::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (<<): {t} and {a}")),
            ),
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs.clone(),
            (rhs, &Value::Bool(b)) => {
                if b {
                    rhs << &Value::Number(Number::Int(1))
                } else {
                    rhs.clone()
                }
            }
            (&Value::Bool(lhs), Value::Number(rhs)) => {
                if lhs {
                    Value::Number(&Number::Int(1) >> rhs)
                } else {
                    Value::Number(Number::Int(0))
                }
            }
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs << rhs),
            (Value::Number(a), Value::String(b)) => {
                let s = Value::String(b.clone());
                match s.to_number() {
                    Err(_) => VMError::UnsupportedOperation(format!("{} << {}", a, b)).to_value(),
                    Ok(r) => Value::Number(a << &r),
                }
            }
            (Value::String(lhs), Value::Number(rhs)) => {
                let lhs = lhs.as_str();
                let s = if rhs.is_negative() {
                    lhs[..=rhs.to_int().unsigned_abs() as usize].to_string()
                } else {
                    lhs[rhs.to_usize().unwrap()..].to_string()
                };
                Value::String(s)
            }
            (Value::String(lhs), Value::String(rhs)) => {
                let mut res = lhs.clone();
                res.push_str(rhs.as_str());
                Value::String(res)
            }
            (Value::Tuple(a), Value::Tuple(b)) => {
                Value::Tuple(a.iter().zip(b).map(|(a, b)| a << b).collect())
            }
            (Value::Tuple(a), b) => Value::Tuple(a.iter().map(|a| a << b).collect()),
            (b, Value::Tuple(a)) => Value::Tuple(a.iter().map(|a| b << a).collect()),
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::define_value_tests;

    define_value_tests! {
        << {
            test_none_shl_none => ((), ()) = ();
            test_none_bool_false_shl_none => (false, ()) = false;
            test_bool_true_shl_none => (true, ()) = true;
            test_none_bool_true_shl_true => ((), true) = ();
            test_false_bool_true_shl_true => (false, true) = 0;
            test_false_0_shl_true => (false, 0) = false;
            test_true_0_shl_true => (true, 0) = 1;
            push_to_end => ("abc", "123") = "abc123";
            int_like => (1, 2.0) = (1 << 2);
            // todo add more test cases
        }
    }
}
