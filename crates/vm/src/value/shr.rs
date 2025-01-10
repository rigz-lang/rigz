use crate::number::Number;
use crate::value::Value;
use crate::VMError;
use std::ops::Shr;

impl Shr for &Value {
    type Output = Value;

    #[inline]
    fn shr(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) | (_, Value::Error(v)) => Value::Error(v.clone()),
            (Value::Type(t), a) | (a, Value::Type(t)) => Value::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (>>): {t} and {a}")),
            ),
            (Value::None, _) => Value::None,
            (rhs, Value::None) => rhs.clone(),
            (rhs, &Value::Bool(b)) => {
                if b {
                    rhs >> &Value::Number(Number::Int(1))
                } else {
                    rhs.clone()
                }
            }
            (&Value::Bool(lhs), Value::Number(rhs)) => {
                if lhs {
                    Value::Number(&Number::Int(1) >> rhs)
                } else {
                    0.into()
                }
            }
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs >> rhs),
            (Value::Number(a), Value::String(b)) => {
                let s = Value::String(b.clone());
                match s.to_number() {
                    Err(_) => VMError::UnsupportedOperation(format!("{} >> {}", a, b)).to_value(),
                    Ok(r) => Value::Number(a >> &r),
                }
            }
            (Value::String(lhs), Value::Number(rhs)) => {
                let lhs = lhs.as_str();
                let s = if rhs.is_negative() {
                    lhs[rhs.to_int().unsigned_abs() as usize..].to_string()
                } else {
                    lhs[..=rhs.to_usize().unwrap()].to_string()
                };
                Value::String(s)
            }
            (Value::String(lhs), Value::String(rhs)) => {
                let mut res = rhs.clone();
                res.push_str(lhs.as_str());
                Value::String(res)
            }
            (Value::Tuple(a), Value::Tuple(b)) => {
                Value::Tuple(a.iter().zip(b).map(|(a, b)| a >> b).collect())
            }
            (Value::Tuple(a), b) => Value::Tuple(a.iter().map(|a| a >> b).collect()),
            (b, Value::Tuple(a)) => Value::Tuple(a.iter().map(|a| b >> a).collect()),
            (lhs, rhs) => {
                VMError::UnsupportedOperation(format!("Not supported: {lhs} >> {rhs}")).into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::define_value_tests;

    define_value_tests! {
        >> {
            test_none_shr_none => ((), ()) = ();
            test_none_bool_false_shr_none => (false, ()) = false;
            test_bool_true_shr_none => (true, ()) = true;
            test_none_bool_true_shr_true => ((), true) = ();
            test_false_bool_true_shr_true => (false, true) = 0;
            test_false_0_shr_true => (false, 0) = 0;
            test_true_0_shr_true => (true, 0) = 1;
            append_to_from => ("abc", "123") = "123abc";
            int_like => (1, 2) = (1 >> 2);
        }
    }
}
