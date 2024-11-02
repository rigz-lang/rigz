use crate::value::Value;
use crate::Logical;

impl Logical<Value> for Value {
    type Output = Value;

    #[inline]
    fn and(self, rhs: Value) -> Self::Output {
        match (self.to_bool(), rhs.to_bool()) {
            (false, _) => self,
            (true, _) => rhs,
        }
    }

    #[inline]
    fn or(self, rhs: Value) -> Self::Output {
        match (self.to_bool(), rhs.to_bool()) {
            (false, _) => rhs,
            (true, _) => self,
        }
    }

    #[inline]
    fn xor(self, rhs: Value) -> Self::Output {
        match (self.to_bool(), rhs.to_bool()) {
            (false, false) | (true, true) => Value::None,
            (false, _) => rhs,
            (true, _) => self,
        }
    }

    #[inline]
    fn elvis(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::None | Value::Error(_), rhs) => rhs,
            (lhs, _) => lhs,
        }
    }
}
