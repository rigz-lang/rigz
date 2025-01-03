use crate::{
    err, errln, out, outln, Clear, Register, Reverse, Unary, UnaryAssign, UnaryOperation, VMError,
    Value, VM,
};
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
    pub fn apply_unary(
        &mut self,
        unary_operation: UnaryOperation,
        val: Rc<RefCell<Value>>,
        output: Register,
    ) {
        let val = eval_unary(unary_operation, val.borrow().deref());
        self.insert_register(output, val.into());
    }

    pub fn handle_unary(&mut self, unary: Unary) {
        let Unary { op, from, output } = unary;
        let val = self.resolve_register(&from);
        self.apply_unary(op, val, output);
    }

    pub fn handle_unary_assign(&mut self, unary: UnaryAssign) {
        let UnaryAssign { op, from } = unary;
        match self.update_register(from, |v| {
            *v = eval_unary(op, v);
            Ok(None)
        }) {
            Ok(_) => {}
            Err(e) => {
                self.insert_register(from, e.into());
            }
        }
    }

    pub fn handle_unary_clear(&mut self, unary: &Unary, clear: &Clear) {
        let Unary { op, from, output } = unary;
        let val = match clear {
            Clear::One(c) if c != from => VMError::RuntimeError(format!(
                "Invalid Register Passed to unary_clear: {} != {}",
                c, from
            ))
            .into(),
            Clear::One(c) => self.remove_register_eval_scope(c),
            c => VMError::RuntimeError(format!("Invalid Option Passed to unary_clear: {:?}", c))
                .into(),
        };
        self.apply_unary(*op, val, *output);
    }
}
