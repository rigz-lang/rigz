use crate::value::Value;
use crate::VMError;
use std::ops::BitAnd;

impl<'vm> BitAnd for Value<'vm> {
    type Output = Value<'vm>;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Error(v), _) => Value::Error(v),
            (_, Value::Error(v)) => Value::Error(v),
            (Value::None, _) => Value::None,
            (_, Value::None) => Value::None,
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(a & b),
            (Value::Bool(a), b) => Value::Bool(a & b.to_bool()),
            (b, Value::Bool(a)) => Value::Bool(a & b.to_bool()),
            (Value::Number(a), Value::Number(b)) => match a & b {
                Ok(n) => Value::Number(n),
                Err(e) => Value::Error(e),
            },
            (Value::Number(a), Value::String(b)) => {
                let s = Value::String(b.clone());
                match s.to_number() {
                    None => VMError::UnsupportedOperation(format!("{} & {}", a, b)).to_value(),
                    Some(r) => match a & r {
                        Ok(n) => Value::Number(n),
                        Err(e) => Value::Error(e),
                    },
                }
            }
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
    use crate::number::Number;
    use crate::value::Value;

    define_value_tests! {
        & {
            test_none_bitand_none => (Value::None, Value::None, Value::None);
            test_none_bool_false_bitand_none => (Value::Bool(false), Value::None, Value::Bool(false));
            test_bool_true_bitand_none => (Value::Bool(true), Value::None, Value::None);
            test_none_bool_true_bitand_true => (Value::None, Value::Bool(true), Value::None);
            test_false_bool_true_bitand_true => (Value::Bool(false), Value::Bool(true), Value::Bool(false));
            test_false_0_bitand_true => (Value::Bool(false), Value::Number(Number::UInt(0)), Value::Bool(false));
            test_true_0_bitand_true => (Value::Bool(true), Value::Number(Number::UInt(0)), Value::Bool(false));
            // bitand more test cases here as needed
        }
    }
}
