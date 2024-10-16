use crate::number::Number;
use crate::value::Value;
use std::ops::Shl;

impl<'vm> Shl for Value<'vm> {
    type Output = Value<'vm>;

    fn shl(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs,
            (rhs, Value::Bool(b)) => {
                if b {
                    rhs << Value::Number(Number::Int(1))
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
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs << rhs),
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
        << {
            test_none_shl_none => (Value::None, Value::None, Value::None);
            test_none_bool_false_shl_none => (Value::Bool(false), Value::None, Value::Bool(false));
            test_bool_true_shl_none => (Value::Bool(true), Value::None, Value::Bool(true));
            test_none_bool_true_shl_true => (Value::None, Value::Bool(true), Value::None);
            test_false_bool_true_shl_true => (Value::Bool(false), Value::Bool(true), Value::Number(Number::zero()));
            test_false_0_shl_true => (Value::Bool(false), Value::Number(Number::UInt(0)), Value::Bool(false));
            test_true_0_shl_true => (Value::Bool(true), Value::Number(Number::UInt(0)), Value::Number(Number::UInt(1)));
            push_to_end => (Value::String("abc".into()), Value::String("123".into()), Value::String("abc123".into()));
            // shl more test cases here as needed
        }
    }
}
