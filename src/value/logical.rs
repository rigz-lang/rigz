use crate::value::Value;
use crate::Logical;

impl<'vm> Logical<Value<'vm>> for Value<'vm> {
    type Output = Value<'vm>;

    #[inline]
    fn and(self, rhs: Value<'vm>) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs,
            (_, rhs) => rhs,
        }
    }

    #[inline]
    fn or(self, rhs: Value<'vm>) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs,
            (_, rhs) => rhs,
        }
    }

    #[inline]
    fn xor(self, rhs: Value<'vm>) -> Self::Output {
        match (self, rhs) {
            (Value::None, _) => Value::None,
            (lhs, Value::None) => lhs,
            (_, rhs) => rhs,
        }
    }
}
