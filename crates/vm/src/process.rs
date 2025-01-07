use crate::{Scope, VMError, Value};
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Process<'vm> {
    pub scope: Scope<'vm>,
    pub channel: (Sender<Value>, Receiver<Value>),
}

impl<'vm> Process<'vm> {
    pub fn new(scope: Scope<'vm>) -> Self {
        Self {
            scope,
            channel: unbounded(),
        }
    }

    pub fn start(&mut self) {}

    pub fn close(&mut self) {}

    pub fn receive(&self, timeout: Option<usize>) -> Value {
        match timeout {
            None => self.channel.1.recv().unwrap_or_else(|e| {
                VMError::RuntimeError(format!("Failed to receive value {e:?}")).into()
            }),
            Some(t) => self
                .channel
                .1
                .recv_timeout(Duration::from_millis(t as u64))
                .unwrap_or_else(|e| {
                    VMError::RuntimeError(format!("`receive` timed out after {t}ms - {e:?}")).into()
                }),
        }
    }

    pub fn send(&self, args: Vec<Value>) -> Result<(), VMError> {
        let v = match args.len() {
            0 => Value::None,
            1 => {
                let mut args = args.into_iter();
                args.next().unwrap()
            }
            _ => args.into(),
        };
        self.channel
            .0
            .send(v)
            .map_err(|e| VMError::RuntimeError(format!("Failed to send - {e:?}")))?;
        Ok(())
    }
}
