use crate::{RigzType, VMError, Value};
use crossbeam::channel::{Receiver, Sender};
use indexmap::IndexMap;

#[derive(Clone, Debug)]
pub enum Message<'vm> {
    Event(&'vm str, Value<'vm>),
    Value(Value<'vm>),
}

#[derive(Clone, Debug)]
pub struct LifecycleDefinition {
    pub scope_id: usize,
}

#[derive(Clone, Debug)]
pub struct Lifecycle<'vm> {
    pub name: String,
    pub definition: IndexMap<RigzType, LifecycleDefinition>,
    pub parent: Option<&'vm Lifecycle<'vm>>,
    pub sender: Sender<Message<'vm>>,
    pub receiver: Receiver<Message<'vm>>,
}

impl<'vm> Lifecycle<'vm> {
    pub(crate) fn run(&mut self) -> () {
        todo!()
    }
    pub(crate) fn send(&mut self, value: Value<'vm>) -> Result<(), VMError> {
        match self.sender.send(Message::Value(value)) {
            Ok(o) => Ok(o),
            Err(e) => Err(VMError::LifecycleError(format!(
                "Failed to send message: {}",
                e
            ))),
        }
    }

    pub(crate) fn send_event(
        &mut self,
        message: &'vm str,
        value: Value<'vm>,
    ) -> Result<(), VMError> {
        match self.sender.send(Message::Event(message, value)) {
            Ok(o) => Ok(o),
            Err(e) => Err(VMError::LifecycleError(format!(
                "Failed to send message: {}",
                e
            ))),
        }
    }
}

impl<'vm> Lifecycle<'vm> {}
