use rigz_ast::*;
use rigz_ast_derive::derive_module;

derive_module! {
    r#"trait VM
        fn mut VM.push(value) -> None
        fn mut VM.peek -> Any?
        fn mut VM.pop -> Any!
    end"#
}

impl RigzVM for VMModule {
    fn mut_vm_push(&self, vm: &mut VM, value: Value) {
        vm.stack.push(value.into());
    }

    fn mut_vm_peek(&self, vm: &mut VM) -> Option<Value> {
        let v = match vm.stack.last() {
            None => return None,
            Some(v) => v.clone(),
        };
        Some(v.resolve(vm).borrow().clone())
    }

    fn mut_vm_pop(&self, vm: &mut VM) -> Result<Value, VMError> {
        let v = vm.next_resolved_value("vm_pop").borrow().clone();
        match v {
            Value::Error(e) => Err(e),
            _ => Ok(v),
        }
    }
}
