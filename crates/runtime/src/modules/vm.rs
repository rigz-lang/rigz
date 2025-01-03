use rigz_ast::*;
use rigz_ast_derive::derive_module;

derive_module!(
    r#"trait VM
        fn mut VM.get_register(register: Number) -> Any!
        fn mut VM.first -> Any!
        fn mut VM.last -> Any!
        fn mut VM.remove_register(register: Number) -> Any!
    end"#
);

impl<'vm> RigzVM<'vm> for VMModule {
    fn mut_vm_get_register(&self, vm: &mut VM<'vm>, register: Number) -> Result<Value, VMError> {
        let u = match register.to_usize() {
            Err(_) => {
                return Err(VMError::UnsupportedOperation(format!(
                    "Invalid argument for vm.get_register, expected non negative number received - {:?}",
                    register
                )))
            }
            Ok(n) => n,
        };
        Ok(vm.resolve_register(&u))
    }

    fn mut_vm_first(&self, vm: &mut VM<'vm>) -> Result<Value, VMError> {
        let first = match vm.registers.first() {
            None => return Err(VMError::EmptyRegister("Registers are empty".to_string())),
            Some((_, v)) => v.borrow().clone(),
        };
        let v = first.resolve(vm);
        Ok(v)
    }

    fn mut_vm_last(&self, vm: &mut VM<'vm>) -> Result<Value, VMError> {
        let last = match vm.registers.last() {
            None => return Err(VMError::EmptyRegister("Registers are empty".to_string())),
            Some((_, v)) => v.borrow().clone(),
        };
        let v = last.resolve(vm);
        Ok(v)
    }

    fn mut_vm_remove_register(&self, vm: &mut VM<'vm>, register: Number) -> Result<Value, VMError> {
        let u = match register.to_usize() {
            Err(_) => {
                return Err(VMError::UnsupportedOperation(format!(
                    "Invalid argument for vm.remove_register, expected non negative number received - {:?}",
                    register
                )))
            }
            Ok(n) => n,
        };
        Ok(vm.remove_register_eval_scope(&u))
    }
}
