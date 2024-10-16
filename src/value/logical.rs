use crate::value::Value;
use crate::Logical;

impl<'vm> Logical<Value<'vm>> for Value<'vm> {
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
