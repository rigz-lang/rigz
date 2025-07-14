use rigz_ast::*;
use rigz_core::*;
use rigz_ast_derive::derive_module;
use rigz_vm::{Runner, VM};

derive_module! {
    r#"trait VM
        fn mut VM.push(value) -> None
        fn mut VM.peek -> Any?
        fn mut VM.pop -> Any!
    end"#
}

impl <'v> RigzVM for VMModule<'v> {
    fn mut_vm_push(&self, vm: &mut VM, value: ObjectValue) {
        vm.stack.push(value.into());
    }

    fn mut_vm_peek(&self, vm: &mut VM) -> Option<ObjectValue> {
        let v = match vm.stack.last() {
            None => return None,
            Some(v) => v.clone(),
        };
        Some(v.resolve(vm).borrow().clone())
    }

    fn mut_vm_pop(&self, vm: &mut VM) -> Result<ObjectValue, VMError> {
        let v = vm.next_resolved_value("vm_pop").borrow().clone();
        match v {
            ObjectValue::Primitive(PrimitiveValue::Error(e)) => Err(e),
            _ => Ok(v),
        }
    }
}
