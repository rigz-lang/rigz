use crate::value::Value;
use crate::Logical;

impl Logical<Value> for Value {
    type Output = Value;

    #[inline]
    fn and(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs,
            (_, rhs) => rhs,
        }
    }

    #[inline]
    fn or(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs,
            (_, rhs) => rhs,
        }
    }

    #[inline]
    fn xor(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs,
            (_, rhs) => rhs,
        }
    }
}
