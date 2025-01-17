use crate::{Number, PrimitiveValue, VMError};
use std::ops::Shr;

impl Shr for &PrimitiveValue {
    type Output = PrimitiveValue;

    #[inline]
    fn shr(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (PrimitiveValue::Error(v), _) | (_, PrimitiveValue::Error(v)) => {
                PrimitiveValue::Error(v.clone())
            }
            (PrimitiveValue::Type(t), a) | (a, PrimitiveValue::Type(t)) => PrimitiveValue::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (>>): {t} and {a}")),
            ),
            (PrimitiveValue::None, _) => PrimitiveValue::None,
            (rhs, PrimitiveValue::None) => rhs.clone(),
            (rhs, &PrimitiveValue::Bool(b)) => {
                if b {
                    rhs >> &PrimitiveValue::Number(Number::Int(1))
                } else {
                    rhs.clone()
                }
            }
            (&PrimitiveValue::Bool(lhs), PrimitiveValue::Number(rhs)) => {
                if lhs {
                    PrimitiveValue::Number(&Number::Int(1) >> rhs)
                } else {
                    0.into()
                }
            }
            (PrimitiveValue::Number(lhs), PrimitiveValue::Number(rhs)) => {
                PrimitiveValue::Number(lhs >> rhs)
            }
            (PrimitiveValue::Number(a), PrimitiveValue::String(b)) => match b.parse() {
                Err(_) => VMError::UnsupportedOperation(format!("{} >> {}", a, b)).to_value(),
                Ok(r) => PrimitiveValue::Number(a >> &r),
            },
            (PrimitiveValue::String(lhs), PrimitiveValue::Number(rhs)) => {
                let lhs = lhs.as_str();
                let s = if rhs.is_negative() {
                    lhs[rhs.to_int().unsigned_abs() as usize..].to_string()
                } else {
                    lhs[..=rhs.to_usize().unwrap()].to_string()
                };
                PrimitiveValue::String(s)
            }
            (PrimitiveValue::String(lhs), PrimitiveValue::String(rhs)) => {
                let mut res = rhs.clone();
                res.push_str(lhs.as_str());
                PrimitiveValue::String(res)
            }
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
