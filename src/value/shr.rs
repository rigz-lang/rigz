use std::ops::Shr;
use crate::value::Value;

impl Shr for Value {
    type Output = Value;

    fn shr(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs >> rhs),
            (Value::String(lhs), Value::Number(rhs)) => {
                let lhs = lhs.as_str();
                let s = if rhs.is_negative() {
                    lhs[rhs.to_int().abs() as usize..].to_string()
                } else {
                    lhs[..=rhs.to_usize().unwrap()].to_string()
                };
                Value::String(s)
            }
            _ => todo!()
        }
    }
}