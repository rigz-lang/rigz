use std::ops::{Rem};
use crate::value::Value;

impl <'vm> Rem for Value<'vm> {
    type Output = Value<'vm>;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) => Value::Error(v),
            (_, Value::Error(v)) => Value::Error(v),
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs,
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(a | b),
            (Value::Bool(a), b) => Value::Bool(a | b.to_bool()),
            (b, Value::Bool(a)) => Value::Bool(a | b.to_bool()),
            (Value::Number(a), Value::Number(b)) => {
                match a % b {
                    Ok(n) => Value::Number(n),
                    Err(e) => Value::Error(e)
                }
            },
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
        % {
            test_none_rem_none => (Value::None, Value::None, Value::None);
            test_none_bool_false_rem_none => (Value::Bool(false), Value::None, Value::Bool(false));
            test_bool_true_rem_none => (Value::Bool(true), Value::None, Value::Bool(true));
            test_none_bool_true_rem_true => (Value::None, Value::Bool(true), Value::None);
            test_false_bool_true_rem_true => (Value::Bool(false), Value::Bool(true), Value::Bool(true));
            test_false_0_rem_true => (Value::Bool(false), Value::Number(Number::UInt(0)), Value::Bool(false));
            test_true_0_rem_true => (Value::Bool(true), Value::Number(Number::UInt(0)), Value::Number(Number::UInt(1)));
            // rem more test cases here as needed
        }
    }
}