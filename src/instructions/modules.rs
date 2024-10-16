use crate::vm::VMState;
use crate::{Register, RigzType, VMError, VM};
use log::error;

impl<'vm> VM<'vm> {
    pub(crate) fn handle_call_module_instruction(
        &mut self,
        module: &'vm str,
        function: &'vm str,
        args: Vec<Register>,
        output: Register,
    ) -> Result<VMState<'vm>, VMError> {
        if self.options.disable_modules {
            error!("Modules are disabled");
            self.insert_register(
                output,
                VMError::UnsupportedOperation(format!(
                    "Modules are disabled: Failed to call module {}.{}",
                    module, function
                ))
                .to_value(),
            );
            return Ok(VMState::Running);
        }
        let f = match self.modules.get(module) {
            None => {
                return Err(VMError::InvalidModule(format!(
                    "Module {} does not exist",
                    module
                )))
            }
            Some(m) => match m.functions.get(function) {
                None => {
                    return Err(VMError::InvalidModuleFunction(format!(
                        "Module {}.{} does not exist",
                        module, function
                    )))
                }
                Some(f) => *f,
            },
        };
        let v = match f(self.get_registers(args)?) {
            Ok(v) => v,
            Err(e) => e.to_value(),
        };
        self.insert_register(output, v);
        Ok(VMState::Running)
    }

    pub fn handle_call_extension_module_instruction(
        &mut self,
        module: &str,
        function: &str,
        this: Register,
        args: Vec<Register>,
        output: Register,
    ) -> Result<VMState<'vm>, VMError> {
        if self.options.disable_modules {
            error!("Modules are disabled");
            self.insert_register(
                output,
                VMError::UnsupportedOperation(format!(
                    "Modules are disabled: Failed to call extension {}::{}.{}",
                    module, this, function
                ))
                .to_value(),
            );
            return Ok(VMState::Running);
        }
        let m = match self.modules.get(module) {
            None => {
                return Err(VMError::InvalidModule(format!(
                    "Module {} does not exist",
                    module
                )))
            }
            Some(m) => m.clone(),
        };
        let this = self.get_register(this)?;
        let rigz_type = this.rigz_type();
        let f = match m.extension_functions.get(&rigz_type) {
            None => match m.extension_functions.get(&RigzType::Any) {
                None => {
                    return Err(VMError::InvalidModuleFunction(format!(
                        "Module {}.{:?} does not exist (Any does not exist)",
                        module, rigz_type
                    )))
                }
                Some(def) => match def.get(function) {
                    None => {
                        return Err(VMError::InvalidModuleFunction(format!(
                            "Module extension {}.{} does not exist",
                            module, function
                        )))
                    }
                    Some(f) => *f,
                },
            },
            Some(def) => match def.get(function) {
                None => {
                    return Err(VMError::InvalidModuleFunction(format!(
                        "Module extension {}.{} does not exist",
                        module, function
                    )))
                }
                Some(f) => f,
            },
        };
        let v = match f(this, self.get_registers(args)?) {
            Ok(o) => o,
            Err(e) => e.to_value(),
        };
        self.insert_register(output, v);
        Ok(VMState::Running)
    }
}
