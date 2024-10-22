use crate::number::Number;
use crate::value::Value;
use crate::VMError;
use std::ops::Shr;

impl Shr for Value {
    type Output = Value;

    #[inline]
    fn shr(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
            (rhs, Value::None) => rhs,
            (rhs, Value::Bool(b)) => {
                if b {
                    rhs >> Value::Number(Number::Int(1))
                } else {
                    rhs
                }
            }
            (Value::Bool(lhs), Value::Number(rhs)) => {
                if lhs {
                    Value::Number(Number::Int(1) >> rhs)
                } else {
                    Value::Number(Number::Int(0))
                }
            }
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs >> rhs),
            (Value::Number(a), Value::String(b)) => {
                let s = Value::String(b.clone());
                match s.to_number() {
                    None => VMError::UnsupportedOperation(format!("{} >> {}", a, b)).to_value(),
                    Some(r) => Value::Number(a >> r),
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
        >> {
            test_none_shr_none => (Value::None, Value::None, Value::None);
            test_none_bool_false_shr_none => (Value::Bool(false), Value::None, Value::Bool(false));
            test_bool_true_shr_none => (Value::Bool(true), Value::None, Value::Bool(true));
            test_none_bool_true_shr_true => (Value::None, Value::Bool(true), Value::None);
            test_false_bool_true_shr_true => (Value::Bool(false), Value::Bool(true), Value::Number(Number::Int(0)));
            test_false_0_shr_true => (Value::Bool(false), Value::Number(Number::Int(0)), Value::Number(Number::Int(0)));
            test_true_0_shr_true => (Value::Bool(true), Value::Number(Number::Int(0)), Value::Number(Number::Int(1)));
            append_to_from => (Value::String("abc".into()), Value::String("123".into()), Value::String("123abc".into()));
            int_like => (Value::Number(1.into()), Value::Number(2.into()), Value::Number((1 >> 2).into()));
            // todo add more test cases
        }
    }
}
