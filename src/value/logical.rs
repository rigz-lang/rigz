use crate::Logical;
use crate::value::Value;

impl Logical<Value> for Value {
    type Output = Value;

    fn and(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs,
            (_, rhs) => rhs,
        }
    }

    fn or(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs,
            (_, rhs) => rhs,
        }
    }

    fn xor(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs,
            (_, rhs) => rhs,
        }
    }
}