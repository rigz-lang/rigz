use rigz_vm::{Module, Register, RegisterValue, VMError, Value, VM};
use std::cell::RefCell;

#[derive(Copy, Clone)]
pub struct VMModule {}

#[allow(unused_variables)]
impl<'vm> Module<'vm> for VMModule {
    fn name(&self) -> &'static str {
        "VM"
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
                Ok(vm.resolve_register(u))
            }
            "first" => {
                let (og, first) = match vm.current.borrow().registers.first() {
                    None => return Err(VMError::EmptyRegister("Registers are empty".to_string())),
                    Some((o, v)) => (*o, v.borrow().clone()),
                };
                let v = resolve_register_value(vm, og, first);
                Ok(v)
            }
            "last" => {
                let (og, last) = match vm.current.borrow().registers.last() {
                    None => return Err(VMError::EmptyRegister("Registers are empty".to_string())),
                    Some((o, v)) => (*o, v.borrow().clone()),
                };
                let v = resolve_register_value(vm, og, last);
                Ok(v)
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
                Ok(vm.remove_register_eval_scope(u))
            }
            f => Err(VMError::UnsupportedOperation(format!(
                "VMModule does not have a function `{}`",
                f
            ))),
        }
    }

    fn trait_definition(&self) -> &'static str {
        r#"trait VM
            fn mut VM.get_register(register: Number) -> Any!
            fn mut VM.first -> Any!
            fn mut VM.last -> Any!
            fn mut VM.remove_register(register: Number) -> Any!
        end"#
    }
}

fn resolve_register_value(vm: &mut VM, index: Register, register_value: RegisterValue) -> Value {
    match register_value {
        RegisterValue::ScopeId(s, out) => vm.handle_scope(s, index, out),
        RegisterValue::Register(r) => vm.resolve_register(r),
        RegisterValue::Value(v) => v,
        RegisterValue::Constant(c) => vm.get_constant(c)
    }
}
