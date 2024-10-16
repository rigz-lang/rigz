use crate::Logical;
use crate::value::Value;

impl <'vm> Logical<Value<'vm>> for Value<'vm> {
    type Output = Value<'vm>;

    fn and(self, rhs: Value<'vm>) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs,
            (_, rhs) => rhs,
        }
    }

    fn or(self, rhs: Value<'vm>) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs,
            (_, rhs) => rhs,
        }
    }

    fn xor(self, rhs: Value<'vm>) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs,
            (_, rhs) => rhs,
        }
    }
}