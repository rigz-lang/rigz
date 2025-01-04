use crate::{err, errln, out, outln, Reverse, UnaryOperation, VMError, Value, VM};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

fn eval_unary(unary_operation: UnaryOperation, val: &Value) -> Value {
    match unary_operation {
        UnaryOperation::Neg => -val,
        UnaryOperation::Not => !val,
        UnaryOperation::PrintLn => {
            outln!("{}", val);
            Value::None
        }
        UnaryOperation::EPrintLn => {
            errln!("{}", val);
            Value::None
        }
        UnaryOperation::Print => {
            out!("{}", val);
            Value::None
        }
        UnaryOperation::EPrint => {
            err!("{}", val);
            Value::None
        }
        UnaryOperation::Reverse => val.reverse(),
    }
}

impl VM<'_> {
    pub fn apply_unary(&mut self, unary_operation: UnaryOperation, val: Rc<RefCell<Value>>) {
        let val = eval_unary(unary_operation, val.borrow().deref());
        self.store_value(val.into());
    }

    pub fn handle_unary(&mut self, op: UnaryOperation) {
        let val = self.next_value("handle_unary");
        self.apply_unary(op, val);
    }
}
