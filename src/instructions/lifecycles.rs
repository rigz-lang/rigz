use crate::vm::VMState;
use crate::{Register, VMError, VM};

impl<'vm> VM<'vm> {
    pub(crate) fn publish(&mut self, value: Register) -> Result<VMState<'vm>, VMError> {
        if self.options.disable_lifecyles {
            return Ok(VMState::Running);
        }

        let value = self.get_register(value)?;
        match self.lifecycles.get_mut("on") {
            None => {
                return Err(VMError::LifecycleError(
                    "Lifecycle does not exist: `on`".to_string(),
                ))
            }
            Some(l) => {
                l.send(value)?;
            }
        }
        Ok(VMState::Running)
    }

    pub fn publish_event(
        &mut self,
        message: &'vm str,
        value: Register,
    ) -> Result<VMState<'vm>, VMError> {
        if self.options.disable_lifecyles {
            return Ok(VMState::Running);
        }

        let value = self.get_register(value)?;
        match self.lifecycles.get_mut("on") {
            None => {
                return Err(VMError::LifecycleError(
                    "Lifecycle does not exist: `on`".to_string(),
                ))
            }
            Some(l) => {
                l.send_event(message, value)?;
            }
        }
        Ok(VMState::Running)
    }
}
