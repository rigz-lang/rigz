use crate::vm::VM;
use crate::Runner;
use rigz_core::{ObjectValue, ResolveValue, VMError};
use std::cell::RefCell;
use std::rc::Rc;

pub enum VMState {
    Running,
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

    #[inline]
    fn handle_scope(&mut self, scope: usize) -> Rc<RefCell<ObjectValue>> {
        let current = self.sp;
        match self.call_frame(scope) {
            Ok(_) => {}
            Err(e) => {
                let o: ObjectValue = e.into();
                return o.into();
            }
        };
        let mut v = match self.run_scope() {
            VMState::Running => unreachable!(),
            VMState::Done(v) => return v,
            VMState::Ran(v) => v,
        };
        while current != self.sp {
            self.stack.push(v.into());
            v = match self.run_scope() {
                VMState::Running => unreachable!(),
                VMState::Done(v) => return v,
                VMState::Ran(v) => v,
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
