use rigz_vm::{Module, VMError, Value, VM};

#[derive(Copy, Clone)]
pub struct VMModule {}

#[allow(unused_variables)]
impl<'vm> Module<'vm> for VMModule {
    fn name(&self) -> &'static str {
        "VM"
    }

    fn call(&self, function: &'vm str, args: Vec<Value>) -> Result<Value, VMError> {
        Err(VMError::UnsupportedOperation(
            "VMModule does not implement `call`".to_string(),
        ))
    }

    fn call_extension(
        &self,
        value: Value,
        function: &'vm str,
        args: Vec<Value>,
    ) -> Result<Value, VMError> {
        Err(VMError::UnsupportedOperation(
            "VMModule does not implement `call_extension`".to_string(),
        ))
    }

    fn vm_extension(
        &self,
        vm: &mut VM<'vm>,
        function: &'vm str,
        args: Vec<Value>,
    ) -> Result<Value, VMError> {
        match function {
            "get_register" => {
                if args.len() != 1 {
                    return Err(VMError::UnsupportedOperation(format!(
                        "Invalid arguments for vm.get_register, expected 1 value received - {:?}",
                        args
                    )));
                }
                let v = args.first().unwrap().clone();
                let u = match v.to_number() {
                    None => {
                        return Err(VMError::UnsupportedOperation(format!(
                            "Invalid argument for vm.get_register, expected number received - {:?}",
                            v
                        )))
                    }
                    Some(n) => match n.to_usize() {
                        Ok(u) => u,
                        Err(e) => return Err(e),
                    },
                };
                vm.get_register(u)
            }
            "remove_register" => {
                if args.len() != 1 {
                    return Err(VMError::UnsupportedOperation(format!(
                        "Invalid arguments for vm.get_register, expected 1 value received - {:?}",
                        args
                    )));
                }
                let v = args.first().unwrap().clone();
                let u = match v.to_number() {
                    None => {
                        return Err(VMError::UnsupportedOperation(format!(
                            "Invalid argument for vm.get_register, expected number received - {:?}",
                            v
                        )))
                    }
                    Some(n) => match n.to_usize() {
                        Ok(u) => u,
                        Err(e) => return Err(e),
                    },
                };
                vm.remove_register(u)
            }
            f => Err(VMError::UnsupportedOperation(format!(
                "VMModule does not have a function `{}`",
                f
            ))),
        }
    }

    fn trait_definition(&self) -> &'static str {
        r#"trait VM
            fn get_register(register: Number) -> Any!
            fn remove_register(register: Number) -> Any!
            fn resolve_register(register: Number) -> Any!
        end"#
    }
}
