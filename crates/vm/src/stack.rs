use crate::{StackValue, VMError};
use derive_more::IntoIterator;
use std::fmt::Display;

#[derive(Debug, Default, IntoIterator)]
pub struct VMStack(Vec<StackValue>);

impl VMStack {
    pub fn new(stack: Vec<StackValue>) -> Self {
        VMStack(stack)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<StackValue> {
        self.0.pop()
    }

    #[inline]
    pub fn push(&mut self, value: StackValue) {
        self.0.push(value)
    }

    pub fn store_value(&mut self, value: StackValue) {
        self.push(value)
    }

    #[inline]
    pub fn last(&self) -> Option<&StackValue> {
        self.0.last()
    }

    pub fn next_value<T: Display>(&mut self, location: T) -> StackValue {
        self.pop()
            .unwrap_or_else(|| VMError::EmptyStack(format!("Stack is empty for {location}")).into())
    }
}
