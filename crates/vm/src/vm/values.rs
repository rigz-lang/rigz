use crate::vm::VM;
use crate::Runner;
use rigz_core::{ObjectValue, ResolveValue, ResolvedValue, VMError};
use std::cell::RefCell;
use std::rc::Rc;

pub enum VMState {
    Running,
    Break,
    Next,
    Done(Rc<RefCell<ObjectValue>>),
    Ran(Rc<RefCell<ObjectValue>>),
}

impl From<VMError> for VMState {
    #[inline]
    fn from(value: VMError) -> Self {
        let o: ObjectValue = value.into();
        VMState::Done(o.into())
    }
}

impl ResolveValue for VM {
    fn location(&self) -> &'static str {
        "VM"
    }

    #[inline] // todo this needs to use VMState, propagation isn't working
    fn handle_scope(&mut self, scope: usize) -> ResolvedValue {
        let current = self.sp;
        if let Err(e) = self.call_frame(scope) {
            let o: ObjectValue = e.into();
            return ResolvedValue::Done(o.into());
        };

        let mut v = match self.run_scope() {
            VMState::Break => return ResolvedValue::Break,
            VMState::Next => return ResolvedValue::Next,
            VMState::Running => unreachable!(),
            VMState::Done(v) => return ResolvedValue::Done(v),
            VMState::Ran(v) => ResolvedValue::Value(v),
        };
        while current != self.frames.current.borrow().scope_id {
            if let ResolvedValue::Value(v) = v {
                self.stack.push(v.into());
            }
            v = match self.run_scope() {
                VMState::Break => return ResolvedValue::Break,
                VMState::Next => return ResolvedValue::Next,
                VMState::Running => unreachable!(),
                VMState::Done(v) => return ResolvedValue::Done(v),
                VMState::Ran(v) => ResolvedValue::Value(v),
            };
        }
        v
    }

    fn get_constant(&self, index: usize) -> Rc<RefCell<ObjectValue>> {
        match self.constants.get(index) {
            None => {
                let o: ObjectValue =
                    VMError::runtime(format!("Constant {index} does not exist")).into();
                o.into()
            }
            Some(v) => v.clone().into(),
        }
    }
}
