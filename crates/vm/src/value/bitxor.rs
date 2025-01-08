use crate::value::Value;
use crate::VMError;
use std::ops::BitXor;

impl BitXor for &Value {
    type Output = Value;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) | (_, Value::Error(v)) => Value::Error(v.clone()),
            (Value::Type(t), a) | (a, Value::Type(t)) => Value::Error(
                VMError::UnsupportedOperation(format!("Invalid Operation (^): {t} and {a}")),
            ),
            (Value::None, Value::None) => Value::None,
            (Value::None, rhs) => rhs.clone(),
            (lhs, Value::None) => lhs.clone(),
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(a ^ b),
            (Value::Bool(a), b) => Value::Bool(a ^ b.to_bool()),
            (b, Value::Bool(a)) => Value::Bool(a ^ b.to_bool()),
            (Value::Number(a), Value::Number(b)) => Value::Number(a ^ b),
            (Value::Number(a), Value::String(b)) => {
                let s = Value::String(b.clone());
                match s.to_number() {
                    Err(_) => VMError::UnsupportedOperation(format!("{} ^ {}", a, b)).to_value(),
                    Ok(r) => Value::Number(a ^ &r),
                }
            }
            (Value::Tuple(a), Value::Tuple(b)) => {
                Value::Tuple(a.iter().zip(b).map(|(a, b)| a ^ b).collect())
            }
            (Value::Tuple(a), b) => Value::Tuple(a.iter().map(|a| a ^ b).collect()),
            (b, Value::Tuple(a)) => Value::Tuple(a.iter().map(|a| b ^ a).collect()),
            // (Value::String(a), Value::String(b)) => {
            //     let mut result = a.clone();
            //     result.push_str(b.as_str());
            //     Value::String(result)
            // }
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
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::define_value_tests;

    define_value_tests! {
        ^ {
            test_none_bitxor_none => ((), ()) = ();
            test_none_bool_false_bitxor_none => (false, ()) = false;
            test_bool_true_bitxor_none => (true, ()) = true;
            test_none_bool_true_bitxor_true => ((), true) = true;
            test_false_bool_true_bitxor_true => (false, true) = true;
            test_false_0_bitxor_true => (false, 0) = false;
            test_true_0_bitxor_true => (true, 0) = 1;
        }
    }
}
