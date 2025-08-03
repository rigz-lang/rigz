use rigz_core::{Snapshot, StackValue, VMError};
use std::fmt::Display;
use std::vec::IntoIter;

#[derive(Debug, Default)]
pub struct VMStack(Vec<StackValue>);

impl Snapshot for VMStack {
    fn as_bytes(&self) -> Vec<u8> {
        self.0.as_bytes()
    }

    fn from_bytes<D: Display>(bytes: &mut IntoIter<u8>, location: &D) -> Result<Self, VMError> {
        Ok(VMStack(Snapshot::from_bytes(bytes, location)?))
    }
}

impl VMStack {
    #[inline]
    pub fn new(stack: Vec<StackValue>) -> Self {
        VMStack(stack)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    #[inline]
    pub fn pop(&mut self) -> Option<StackValue> {
        self.0.pop()
    }

    #[inline]
    pub fn push(&mut self, value: StackValue) {
        self.0.push(value)
    }

    #[inline]
    pub fn store_value(&mut self, value: StackValue) {
        self.push(value)
    }

    #[inline]
    pub fn last(&self) -> Option<&StackValue> {
        self.0.last()
    }

    #[inline]
    pub fn next_value<T: Display, F>(&mut self, location: F) -> StackValue
    where
        F: FnOnce() -> T,
    {
        self.pop().unwrap_or_else(|| {
            VMError::EmptyStack(format!("Stack is empty for {}", location())).into()
        })
    }
}
