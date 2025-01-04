use crate::{BinaryOperation, Logical, Value, VM};
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

#[inline]
fn eval_binary_operation(binary_operation: BinaryOperation, lhs: &Value, rhs: &Value) -> Value {
    match binary_operation {
        BinaryOperation::Add => lhs + rhs,
        BinaryOperation::Sub => lhs - rhs,
        BinaryOperation::Shr => lhs >> rhs,
        BinaryOperation::Shl => lhs << rhs,
        BinaryOperation::Eq => Value::Bool(lhs == rhs),
        BinaryOperation::Neq => Value::Bool(lhs != rhs),
        BinaryOperation::Mul => lhs * rhs,
        BinaryOperation::Div => lhs / rhs,
        BinaryOperation::Rem => lhs % rhs,
        BinaryOperation::BitOr => lhs | rhs,
        BinaryOperation::BitAnd => lhs & rhs,
        BinaryOperation::BitXor => lhs ^ rhs,
        BinaryOperation::And => lhs.and(rhs),
        BinaryOperation::Or => lhs.or(rhs),
        BinaryOperation::Xor => lhs.xor(rhs),
        BinaryOperation::Gt => Value::Bool(lhs > rhs),
        BinaryOperation::Gte => Value::Bool(lhs >= rhs),
        BinaryOperation::Lt => Value::Bool(lhs < rhs),
        BinaryOperation::Lte => Value::Bool(lhs <= rhs),
        BinaryOperation::Elvis => lhs.elvis(rhs),
    }
}

impl VM<'_> {
    #[inline]
    pub fn apply_binary(
        &mut self,
        binary_operation: BinaryOperation,
        lhs: Rc<RefCell<Value>>,
        rhs: Rc<RefCell<Value>>,
    ) {
        let v = eval_binary_operation(binary_operation, lhs.borrow().deref(), rhs.borrow().deref());
        self.store_value(v.into())
    }

    #[inline]
    pub fn handle_binary(&mut self, op: BinaryOperation) {
        let rhs = self.next_value("handle_binary - rhs");
        let lhs = self.next_value("handle_binary - lhs");
        self.apply_binary(op, lhs, rhs);
    }

    pub fn handle_binary_assign(&mut self, op: BinaryOperation) {
        let rhs = self.next_value("handle_binary_assign - rhs");
        let lhs = self.next_value("handle_binary_assign - lhs");
        let v = eval_binary_operation(op, lhs.borrow().deref(), rhs.borrow().deref());
        *lhs.borrow_mut().deref_mut() = v;
        self.store_value(lhs.into())
    }
}
