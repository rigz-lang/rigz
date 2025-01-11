use crate::{StackValue, VMError};
use std::fmt::Display;

#[derive(Debug, Default)]
pub struct VMStack(Vec<StackValue>);

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
    pub fn next_value<T: Display>(&mut self, location: T) -> StackValue {
        self.pop()
            .unwrap_or_else(|| VMError::EmptyStack(format!("Stack is empty for {location}")).into())
    }
}
