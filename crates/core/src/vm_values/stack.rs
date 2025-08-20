use crate::ObjectValue;
use std::cell::RefCell;
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

#[derive(Debug)]
pub enum ResolvedValue {
    Break,
    Next,
    Done(Rc<RefCell<ObjectValue>>),
    Value(Rc<RefCell<ObjectValue>>),
}

impl ResolvedValue {
    pub fn unwrap_or_default(self) -> Rc<RefCell<ObjectValue>> {
        let Self::Value(v) = self else {
            return ObjectValue::default().into();
        };
        v
    }
}

pub trait ResolveValue {
    fn location(&self) -> &'static str;

    fn handle_scope(&mut self, scope: usize) -> ResolvedValue;

    fn get_constant(&self, constant_id: usize) -> Rc<RefCell<ObjectValue>>;
}

impl StackValue {
    pub fn resolve<T: ResolveValue + ?Sized>(&self, vm: &mut T) -> Rc<RefCell<ObjectValue>> {
        match self {
            &StackValue::ScopeId(scope) => vm.handle_scope(scope).unwrap_or_default(),
            StackValue::Value(v) => v.clone(),
            &StackValue::Constant(c) => vm.get_constant(c),
        }
    }
}
