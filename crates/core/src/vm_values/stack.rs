use crate::ObjectValue;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StackValue {
    ScopeId(usize),
    Value(Rc<RefCell<ObjectValue>>),
    Constant(usize),
}

impl<T: Into<ObjectValue>> From<T> for StackValue {
    fn from(value: T) -> Self {
        let v: ObjectValue = value.into();
        StackValue::Value(v.into())
    }
}

impl From<Rc<RefCell<ObjectValue>>> for StackValue {
    #[inline]
    fn from(value: Rc<RefCell<ObjectValue>>) -> Self {
        StackValue::Value(value)
    }
}

pub trait ResolveValue {
    fn location(&self) -> &'static str;

    fn handle_scope(&mut self, scope: usize) -> Rc<RefCell<ObjectValue>>;

    fn get_constant(&self, constant_id: usize) -> Rc<RefCell<ObjectValue>>;
}

impl StackValue {
    pub fn resolve<T: ResolveValue + ?Sized>(&self, vm: &mut T) -> Rc<RefCell<ObjectValue>> {
        match self {
            &StackValue::ScopeId(scope) => vm.handle_scope(scope),
            StackValue::Value(v) => v.clone(),
            &StackValue::Constant(c) => vm.get_constant(c),
        }
    }
}
