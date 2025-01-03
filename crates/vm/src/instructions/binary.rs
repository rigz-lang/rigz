use crate::{Binary, BinaryAssign, BinaryOperation, Clear, Logical, Register, VMError, Value, VM};
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
        output: Register,
    ) {
        let v = eval_binary_operation(binary_operation, lhs.borrow().deref(), rhs.borrow().deref());

        self.insert_register(output, v.into());
    }

    #[inline]
    pub fn handle_binary(&mut self, binary: Binary) {
        let Binary {
            op,
            lhs,
            rhs,
            output,
        } = binary;
        let lhs = self.resolve_register(&lhs);
        let rhs = self.resolve_register(&rhs);
        self.apply_binary(op, lhs, rhs, output);
    }

    pub fn handle_binary_assign(&mut self, binary: BinaryAssign) {
        let BinaryAssign { op, lhs, rhs } = binary;
        match self.update_register(lhs, &[rhs], |v, args| {
            let rhs = args[0].clone();
            let res = eval_binary_operation(op, v.borrow().deref(), rhs.borrow().deref());
            *v.borrow_mut().deref_mut() = res;
            Ok(None)
        }) {
            Ok(_) => {}
            Err(e) => {
                self.insert_register(lhs, e.into());
            }
        };
    }

    pub fn handle_binary_clear(&mut self, binary: &Binary, clear: &Clear) {
        let Binary {
            op,
            lhs,
            rhs,
            output,
        } = binary;
        let (lhs, rhs) = match clear {
            Clear::One(c) if c == rhs => (
                self.resolve_register(lhs),
                self.remove_register_eval_scope(c),
            ),
            Clear::One(c) if c == lhs => (
                self.remove_register_eval_scope(c),
                self.resolve_register(rhs),
            ),
            Clear::Two(c1, c2) if c1 == lhs && c2 == rhs => (
                self.remove_register_eval_scope(c1),
                self.remove_register_eval_scope(c2),
            ),
            Clear::Two(c1, c2) if c2 == lhs && c1 == rhs => (
                self.remove_register_eval_scope(c2),
                self.remove_register_eval_scope(c1),
            ),
            Clear::One(c) => (
                self.remove_register_eval_scope(c),
                VMError::RuntimeError(format!(
                    "Invalid Register Passed to binary_clear: {} must be {} or {}",
                    c, lhs, rhs
                ))
                .into(),
            ),
            Clear::Two(c1, c2) => {
                let v: Rc<RefCell<Value>> = VMError::RuntimeError(format!(
                    "Invalid Registers Passed to binary_clear: {} and {} must be either {} or {}",
                    c1, c2, lhs, rhs
                ))
                .to_value()
                .into();
                (v.clone(), v)
            }
            c => {
                let v: Rc<RefCell<Value>> = VMError::RuntimeError(format!(
                    "Invalid Option Passed to binary_clear: {:?}",
                    c
                ))
                .to_value()
                .into();
                (v.clone(), v)
            }
        };
        self.apply_binary(*op, lhs, rhs, *output);
    }
}
