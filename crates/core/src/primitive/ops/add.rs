use crate::{PrimitiveValue, VMError};
use std::ops::Add;

impl Add for &PrimitiveValue {
    type Output = PrimitiveValue;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (PrimitiveValue::Error(v), _) | (_, PrimitiveValue::Error(v)) => v.into(),
            (PrimitiveValue::Type(t), a) | (a, PrimitiveValue::Type(t)) => PrimitiveValue::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (+): {t} and {a}")),
            ),
            (PrimitiveValue::None, v) | (v, PrimitiveValue::None) => v.clone(),
            (PrimitiveValue::Bool(a), PrimitiveValue::Bool(b)) => PrimitiveValue::Bool(a | b),
            (PrimitiveValue::Number(a), PrimitiveValue::Number(b)) => PrimitiveValue::Number(a + b),
            (PrimitiveValue::Number(a), PrimitiveValue::String(b)) => match b.parse() {
                Err(_) => {
                    let mut res = a.to_string();
                    res.push_str(b.as_str());
                    PrimitiveValue::String(res)
                }
                Ok(r) => PrimitiveValue::Number(a + &r),
            },
            (PrimitiveValue::String(a), PrimitiveValue::Number(b)) => match a.parse() {
                Err(_) => {
                    let mut res = a.to_string();
                    res.push_str(b.to_string().as_str());
                    PrimitiveValue::String(res)
                }
                Ok(r) => PrimitiveValue::Number(b + &r),
            },
            (PrimitiveValue::Number(a), PrimitiveValue::Range(r))
            | (PrimitiveValue::Range(r), PrimitiveValue::Number(a)) => match r + a {
                None => VMError::UnsupportedOperation(format!("Unable to add {a} to {r}")).into(),
                Some(r) => PrimitiveValue::Range(r),
            },
            (PrimitiveValue::Range(a), PrimitiveValue::Range(b)) => match a + b {
                None => {
                    VMError::UnsupportedOperation(format!("Unable to add ranges: {a} + {b}")).into()
                }
                Some(r) => PrimitiveValue::Range(r),
            },
            (PrimitiveValue::Range(a), PrimitiveValue::String(b))
            | (PrimitiveValue::String(b), PrimitiveValue::Range(a)) => {
                VMError::UnsupportedOperation(format!("Cannot perform {a} + {b}")).into()
            }
            (PrimitiveValue::String(a), PrimitiveValue::String(b)) => {
                let mut result = a.clone();
                result.push_str(b.as_str());
                PrimitiveValue::String(result)
            }
            // todo should "a" + true = "atrue" or true
            (PrimitiveValue::Bool(a), b) | (b, PrimitiveValue::Bool(a)) => {
                PrimitiveValue::Bool(a | b.to_bool())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::define_value_tests;

    define_value_tests! {
        + {
            test_none_add_none => ((), ()) = ();
            test_none_bool_false_add_none => (false, ()) = false;
            test_bool_true_add_none => (true, ()) = true;
            test_none_bool_true_add_true => ((), true) = true;
            test_false_bool_true_add_true => (false, true) = true;
            test_false_0_add_true => (false, 0) = false;
            test_true_0_add_true => (true, 0) = 1;
            test_str_1_add_num => ("1", 6) = 7;
            test_str_abc_add_num => ("abc", 6) = "abc6";
            test_num_add_abc => (6, "abc") = "6abc";
        }
    }
}
