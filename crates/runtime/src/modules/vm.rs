use rigz_ast::*;
use rigz_ast_derive::derive_module;

derive_module!(
    r#"trait VM
        fn mut VM.push(value) -> None
        fn mut VM.peek -> Any?
        fn mut VM.pop -> Any!
    end"#
);

impl<'vm> RigzVM<'vm> for VMModule {
    fn mut_vm_push(&self, vm: &mut VM<'vm>, value: Value) {
        vm.stack.push(value.into());
    }

    fn mut_vm_peek(&self, vm: &mut VM<'vm>) -> Option<Value> {
        let v = match vm.stack.last() {
            None => return None,
            Some(v) => v.clone(),
        };
        Some(v.resolve(vm).borrow().clone())
    }

    fn mut_vm_pop(&self, vm: &mut VM<'vm>) -> Result<Value, VMError> {
        let v = vm.next_value("vm_pop").borrow().clone();
        match v {
            Value::Error(e) => Err(e),
            _ => Ok(v),
        }
    }
}
