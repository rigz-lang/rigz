use std::ops::Shl;
use crate::value::Value;

impl Shl for Value {
    type Output = Value;

    fn shl(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
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
            _ => todo!()
        }
    }
}